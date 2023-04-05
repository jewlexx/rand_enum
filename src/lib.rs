#![doc = include_str!("../README.md")]
#![warn(missing_docs, clippy::all, clippy::pedantic)]

#[macro_use]
extern crate proc_macro_error;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro_crate::FoundCrate;
use syn::{spanned::Spanned, DeriveInput, LitInt};

/// Implements [`rand::distributions::Distribution`](https://rust-random.github.io/rand/rand/distributions/distribution/trait.Distribution.html) for a given enum
///
/// Information on weights is as follows. Note that this is not much, and much more information can be found at [`rand::distributions::WeightedIndex`](https://rust-random.github.io/rand/rand/distributions/struct.WeightedIndex.html)
/// - According to rand, a weight cannot be negative
/// - 0: A weight of `0` means it will never be selected
/// - 1: A weight of `1` is the default weight, and is thus redundant
/// - The higher the weight is, the more likely it will be selected
/// - If a type is specified, (i.e 1_u8), that same type must be obeyed for all the other weights
/// - By default the type is i32, as this is Rust's default number type
/// - Weights cannot all be `0`, as there will be no value chosen, ever
#[proc_macro_error]
#[proc_macro_derive(Distribution, attributes(weight))]
pub fn derive_distribute(input: TokenStream) -> TokenStream {
    let rand_crate = match proc_macro_crate::crate_name("rand") {
        Ok(found) => {
            let name = match found {
                FoundCrate::Itself => "crate".to_string(),
                FoundCrate::Name(name) => name,
            };

            Ident::new(&name, name.span())
        }
        Err(_) => abort_call_site!("could not find `rand` crate"),
    };

    let ast: DeriveInput = syn::parse(input).unwrap();
    let enum_ident = &ast.ident;

    let mut variants: Vec<(Ident, LitInt)> = vec![];

    match ast.data {
        syn::Data::Enum(data) => {
            for var in &data.variants {
                let mut variant_weight = None;
                for attr in &var.attrs {
                    match &attr.meta {
                        syn::Meta::List(meta) =>
                        {
                            #[allow(clippy::manual_let_else)]
                            if meta.path.is_ident("weight") {
                                let weight = match meta.parse_args::<LitInt>() {
                                    Ok(weight) => weight,
                                    _ => {
                                        abort!(
                                            meta.tokens.span(),
                                            "could not parse weight. expected a integer literal"
                                        );
                                    }
                                };

                                if let Ok(weight_value) = weight.base10_parse::<u128>() {
                                    if weight_value == 1 {
                                        emit_warning!(
                                            meta.tokens.span(),
                                            "weight of 1 is not recommended. this is the default value and will effectively be ignored"
                                        );
                                    }
                                }

                                variant_weight = Some(weight);
                            }
                        }
                        _ => {
                            abort!(
                                attr.span(),
                                "`#[weight]` attribute requires an integer argument"
                            )
                        }
                    }
                }

                variants.push((
                    var.ident.clone(),
                    variant_weight.unwrap_or_else(|| LitInt::new("1", "1".span())),
                ));
            }
        }
        _ => abort_call_site!("should only be derived for enums"),
    }

    let (variant, weight): (Vec<Ident>, Vec<LitInt>) = variants.into_iter().unzip();

    if weight.iter().all(|weight| {
        let parsed: u128 = weight.base10_parse().unwrap();
        parsed == 0
    }) {
        abort_call_site!("all weights are zero");
    }

    // todo!()

    quote! {
        impl #rand_crate::distributions::Distribution<Colours> for #rand_crate::distributions::Standard {
            fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Colours {
                let mut items = vec![
                    #((#enum_ident::#variant, #weight)),*
                ];
                let weight_dist = #rand_crate::distributions::WeightedIndex::new(items.iter().map(|(_, weight)| weight)).unwrap();

                let item = items.swap_remove(weight_dist.sample(rng));

                item.0
            }
        }
    }.into()
}
