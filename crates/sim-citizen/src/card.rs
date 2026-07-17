//! Builds the browse Card record for a registered citizen.

use sim_kernel::{Cx, Result, Symbol, Value};

use crate::{CitizenInfo, NonCitizenInfo, parse_symbol};

/// Builds the browse Card record (a `core` table) for one registered citizen.
///
/// The record carries the citizen's symbol, version, owning crate, and arity,
/// rendered through `cx`'s factory. The kernel defines `Value`/`Symbol` and the
/// factory surface; this helper only assembles the row from a [`CitizenInfo`].
pub fn citizen_card(cx: &mut Cx, info: &CitizenInfo) -> Result<Value> {
    cx.factory().table(vec![
        (
            Symbol::new("symbol"),
            cx.factory().symbol(parse_symbol(info.symbol))?,
        ),
        (
            Symbol::new("version"),
            cx.factory()
                .number_literal(parse_symbol("citizen/int"), info.version.to_string())?,
        ),
        (
            Symbol::new("crate"),
            cx.factory().string(info.crate_name.to_owned())?,
        ),
        (
            Symbol::new("arity"),
            cx.factory()
                .number_literal(parse_symbol("citizen/int"), info.arity.to_string())?,
        ),
    ])
}

/// Builds the browse Card record (a `core` table) for one non-citizen
/// exemption.
///
/// The record carries the exempt type's name, owning crate, reason, kind, and
/// descriptor strategy, rendered through `cx`'s factory.
pub fn non_citizen_card(cx: &mut Cx, info: &NonCitizenInfo) -> Result<Value> {
    cx.factory().table(vec![
        (
            Symbol::new("type_name"),
            cx.factory().string(info.type_name.to_owned())?,
        ),
        (
            Symbol::new("crate"),
            cx.factory().string(info.crate_name.to_owned())?,
        ),
        (
            Symbol::new("reason"),
            cx.factory().string(info.reason.to_owned())?,
        ),
        (
            Symbol::new("kind"),
            cx.factory().string(info.kind.to_owned())?,
        ),
        (
            Symbol::new("descriptor"),
            cx.factory().string(info.descriptor.to_owned())?,
        ),
    ])
}
