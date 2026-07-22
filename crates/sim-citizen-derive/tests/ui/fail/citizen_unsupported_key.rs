include!("../support/citizen_shim.rs");

use sim_citizen_derive::Citizen;

#[derive(Clone, Debug, Default, PartialEq, Citizen)]
#[citizen(symbol = "example/UnsupportedKey", version = 1, extra = "unexpected")]
struct UnsupportedKey {
    value: i64,
}

fn main() {}
