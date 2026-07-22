include!("../support/citizen_shim.rs");

use sim_citizen_derive::Citizen;

#[derive(Clone, Debug, PartialEq, Citizen)]
#[citizen(symbol = "example/MissingDefault", version = 1)]
struct MissingDefault {
    value: i64,
}

fn main() {}
