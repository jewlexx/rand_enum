#[macro_use]
extern crate proc_macro_error;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro_crate::FoundCrate;
use syn::{spanned::Spanned, DeriveInput, LitInt};

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
                        syn::Meta::List(meta) => {
                            if meta.path.is_ident("weight") {
                                let Ok(weight) = meta.parse_args::<LitInt>() else {
                                    abort!(meta.tokens.span(), "could not parse weight. expected a integer literal");
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
