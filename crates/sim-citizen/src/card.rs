//! Builds the browse Card record for a registered citizen.

use sim_kernel::{Cx, Ref, Result, Symbol, Value, card::card_for_ref_with_fallback};

use crate::{CitizenInfo, NonCitizenInfo, parse_symbol};

/// Builds the browse Card record for one registered citizen.
///
/// The record is a real kernel `Card` with the fixed predicate schema plus
/// citizen-specific metadata such as the owning crate, version, arity, and any
/// loaded class members.
pub fn citizen_card(cx: &mut Cx, info: &CitizenInfo) -> Result<Value> {
    let subject = parse_symbol(info.symbol);
    let mut entries = vec![
        (
            Symbol::new("help"),
            cx.factory()
                .string("citizen class constructor surface".to_owned())?,
        ),
        (
            Symbol::new("kind"),
            cx.factory().symbol(Symbol::qualified("core", "class"))?,
        ),
        (
            Symbol::new("args"),
            cx.factory().symbol(Symbol::qualified("core", "Any"))?,
        ),
        (
            Symbol::new("result"),
            cx.factory().symbol(Symbol::qualified("core", "Any"))?,
        ),
        (Symbol::new("shape-known"), cx.factory().bool(false)?),
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
    ];
    append_loaded_members(cx, &mut entries, &subject)?;
    let fallback = cx.factory().table(entries)?;
    card_for_ref_with_fallback(
        cx,
        Ref::Symbol(subject),
        Some(fallback),
        Some(Symbol::qualified("core", "class")),
    )
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

fn append_loaded_members(
    cx: &mut Cx,
    entries: &mut Vec<(Symbol, Value)>,
    subject: &Symbol,
) -> Result<()> {
    let Some(class_value) = cx.registry().class_by_symbol(subject).cloned() else {
        return Ok(());
    };
    let Some(class) = class_value.object().as_class() else {
        return Ok(());
    };
    let members = class.members(cx)?;
    let Some(table) = members.object().as_table_impl() else {
        return Ok(());
    };
    for (key, value) in table.entries(cx)? {
        upsert_entry(entries, key, value);
    }
    Ok(())
}

fn upsert_entry(entries: &mut Vec<(Symbol, Value)>, key: Symbol, value: Value) {
    if let Some((_, slot)) = entries.iter_mut().find(|(existing, _)| *existing == key) {
        *slot = value;
    } else {
        entries.push((key, value));
    }
}
