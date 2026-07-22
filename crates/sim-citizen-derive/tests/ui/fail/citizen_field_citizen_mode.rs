include!("../support/citizen_shim.rs");

use sim_citizen_derive::Citizen;

#[derive(Clone, Debug, Default, PartialEq, Citizen)]
#[citizen(symbol = "example/NestedFieldMode", version = 1)]
struct NestedFieldMode {
    #[citizen(citizen)]
    value: i64,
}

fn main() {}
