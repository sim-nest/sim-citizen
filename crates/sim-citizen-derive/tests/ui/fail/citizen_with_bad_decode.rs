include!("../support/citizen_shim.rs");

use sim_citizen_derive::Citizen;

#[derive(Clone, Debug, Default, PartialEq, Citizen)]
#[citizen(symbol = "example/BadDecodeCodec", version = 1)]
struct BadDecodeCodec {
    #[citizen(with = bad_decode_codec)]
    value: i64,
}

mod bad_decode_codec {
    use super::{Expr, Result};

    pub fn encode(value: &i64) -> Expr {
        Expr::String(value.to_string())
    }

    pub fn decode(_expr: Expr) -> Result<i64> {
        Ok(0)
    }
}

fn main() {}
