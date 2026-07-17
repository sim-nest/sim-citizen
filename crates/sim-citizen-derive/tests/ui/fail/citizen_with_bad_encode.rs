include!("../support/citizen_shim.rs");

use sim_citizen_derive::Citizen;

#[derive(Clone, Debug, Default, PartialEq, Citizen)]
#[citizen(symbol = "example/BadEncodeCodec", version = 1)]
struct BadEncodeCodec {
    #[citizen(with = bad_encode_codec)]
    value: i64,
}

mod bad_encode_codec {
    use super::{Expr, Result};

    pub fn encode(_value: i64) -> Expr {
        Expr::String("0".to_owned())
    }

    pub fn decode(_expr: &Expr) -> Result<i64> {
        Ok(0)
    }
}

fn main() {}
