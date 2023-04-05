use rand::{thread_rng, Rng};
use rand_enum::Distribution;

#[derive(Debug, Distribution)]
enum Colours {
    #[weight(1)]
    Red,
    #[weight(0)]
    Green,
    #[weight(0)]
    Blue,
}

#[test]
fn test_get_rand_colour() {
    let mut rng = thread_rng();
    let colour: Colours = rng.gen();

    dbg!(colour);
}
