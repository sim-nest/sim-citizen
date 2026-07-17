include!("../support/citizen_shim.rs");

use sim_citizen_derive::Citizen;

#[derive(Clone, Debug, Default, PartialEq, Citizen)]
#[citizen(symbol = "example/ListFieldMode", version = 1)]
struct ListFieldMode {
    #[citizen(list)]
    value: i64,
}

fn main() {}
