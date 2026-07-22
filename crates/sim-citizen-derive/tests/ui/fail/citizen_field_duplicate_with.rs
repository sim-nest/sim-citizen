include!("../support/citizen_shim.rs");

use sim_citizen_derive::Citizen;

#[derive(Clone, Debug, Default, PartialEq, Citizen)]
#[citizen(symbol = "example/DuplicateWith", version = 1)]
struct DuplicateWith {
    #[citizen(with = first_codec, with = second_codec)]
    value: i64,
}

mod first_codec {
    use super::{Expr, Result};

    pub fn encode(value: &i64) -> Expr {
        Expr::String(value.to_string())
    }

    pub fn decode(_expr: &Expr) -> Result<i64> {
        Ok(0)
    }
}

mod second_codec {
    use super::{Expr, Result};

    pub fn encode(value: &i64) -> Expr {
        Expr::String(value.to_string())
    }

    pub fn decode(_expr: &Expr) -> Result<i64> {
        Ok(0)
    }
}

fn main() {}
