include!("../support/citizen_shim.rs");

use sim_citizen_derive::Citizen;

#[derive(Clone, Debug, Default, PartialEq, Citizen)]
#[citizen(symbol = "example/DuplicateSymbol", symbol = "example/Other", version = 1)]
struct DuplicateSymbol {
    value: i64,
}

fn main() {}
