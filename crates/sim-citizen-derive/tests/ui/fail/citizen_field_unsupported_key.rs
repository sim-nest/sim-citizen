include!("../support/citizen_shim.rs");

use sim_citizen_derive::Citizen;

#[derive(Clone, Debug, Default, PartialEq, Citizen)]
#[citizen(symbol = "example/UnsupportedFieldKey", version = 1)]
struct UnsupportedFieldKey {
    #[citizen(extra = "unexpected")]
    value: i64,
}

fn main() {}
