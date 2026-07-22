//! Citizen support outside the SIM kernel.
//!
//! A citizen is a public SIM-facing runtime value with a class-backed read
//! constructor, constructor encoding, conformance fixture, and census row.
//! Domain values usually opt in with `#[derive(Citizen)]`; hard cases may
//! register hand-written citizens, and live handles carry inline
//! `#[non_citizen(reason = "...", kind = "...", descriptor = "...")]`
//! exemptions that name their descriptor strategy.
//!
//! This crate owns only the shared support layer: registry rows, runtime
//! installation helpers, fixture checks, generated census rendering, and the
//! semantic equality helpers used by the strict citizen gate. Read-construct
//! remains capability-gated by the codec/runtime path, not by this crate.
//!
//! # Surface
//!
//! Conformance fixtures check a citizen's read-construct round trip; the field
//! and equality traits encode citizen fields and back the strict semantic
//! equality gate; the registry and runtime helpers install citizens into a
//! library and a context; the card and census helpers render browse and census
//! output for both citizens and explicit non-citizen exemptions; and a
//! reference citizen value is provided as an example.
//!
//! # Explicit registration
//!
//! Inventory-backed discovery is convenient for normal host binaries. Strict
//! release, LTO, and wasm checks can instead build a [`CitizenRegistry`] by
//! calling [`CitizenRegistry::register`] for each expected citizen type, load
//! that registry as a kernel library, and call
//! [`run_registry_conformance_expecting`]. The expected-symbol guard reports a
//! missing row instead of letting a shorter registry pass.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

extern crate self as sim_citizen;

mod card;
mod census;
mod conformance;
mod eq;
mod field;
mod read_construct;
mod registry;
mod runtime;
mod symbol;

pub use ::inventory;
pub use card::{citizen_card, non_citizen_card};
pub use census::{
    citizen_census_markdown, citizen_registry_census_markdown, non_citizen_census_markdown,
    render_citizen_census, render_non_citizen_census,
};
pub use conformance::{
    check_default_fixture, check_fixture, check_value_fixture,
    check_value_fixture_with_wrong_version, run_registered_conformance,
    run_registered_conformance_expecting, run_registry_conformance,
    run_registry_conformance_expecting,
};
pub use eq::{CitizenEq, expr_citizen_eq, values_citizen_eq};
pub use field::{
    CitizenField, arity_error, decode_version, field_error, value_from_expr, value_to_expr,
};
pub use read_construct::text_read_construct_expr;
pub use registry::{
    CitizenInfo, CitizenLib, CitizenRegistry, InstallFn, NonCitizenInfo, install_all,
    install_namespace, registered_citizens, registered_non_citizens,
};
pub use runtime::{Citizen, CitizenRuntime, constructor_expr, install_derived};
pub use symbol::parse_symbol;

#[cfg(test)]
mod example;

#[cfg(test)]
mod tests;
