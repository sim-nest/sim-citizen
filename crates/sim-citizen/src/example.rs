//! An example #[derive(Citizen)] value used as a fixture and reference.

use sim_citizen_derive::Citizen;

/// Reference citizen: a two-field point used as a fixture and derive example.
#[derive(Clone, Debug, Default, PartialEq, Citizen)]
#[citizen(symbol = "example/Point", version = 1)]
pub struct Point {
    /// The x coordinate.
    pub x: i64,
    /// The y coordinate.
    pub y: i64,
}
