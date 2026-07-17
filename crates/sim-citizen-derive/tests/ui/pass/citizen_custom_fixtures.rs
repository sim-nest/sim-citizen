include!("../support/citizen_shim.rs");

use sim_citizen_derive::Citizen;

#[derive(Clone, Debug, PartialEq, Citizen)]
#[citizen(
    symbol = "example/FixtureCase",
    version = 1,
    example = fixture_case_example,
    fixtures = fixture_case_fixtures
)]
struct FixtureCase {
    #[citizen(with = fixture_codec)]
    value: i64,
}

fn fixture_case_example() -> FixtureCase {
    FixtureCase { value: 7 }
}

fn fixture_case_fixtures() -> [FixtureCase; 2] {
    [FixtureCase { value: -1 }, FixtureCase { value: 42 }]
}

mod fixture_codec {
    use super::{Expr, Result};

    pub fn encode(value: &i64) -> Expr {
        Expr::String(value.to_string())
    }

    pub fn decode(_expr: &Expr) -> Result<i64> {
        Ok(0)
    }
}

fn main() {
    let _ = FixtureCase::citizen_fields();
}
