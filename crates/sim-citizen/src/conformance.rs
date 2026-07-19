//! Conformance fixtures that check a citizen's read-construct round trip.

use std::sync::Arc;

use sim_kernel::{
    CapabilitySet, Cx, Error, Expr, ObjectEncoding, Result, Symbol, Value,
    read_construct_capability,
};

use crate::{
    CitizenInfo, CitizenLib, CitizenRegistry, CitizenRuntime, field_error, value_from_expr,
    values_citizen_eq,
};

/// Loads every registered citizen and runs each citizen's conformance fixture.
///
/// Installs [`CitizenLib::all`] into `cx`, then invokes the recorded
/// conformance check for each registered `CitizenInfo`. This is the entry
/// point a test harness calls to assert the whole registered set conforms.
pub fn run_registered_conformance(cx: &mut Cx) -> Result<()> {
    let registry = CitizenRegistry::from_inventory()?;
    cx.load_lib(&CitizenLib::all())?;
    run_conformance_rows(cx, registry.citizens())
}

/// Runs inventory-backed conformance after checking expected symbols.
///
/// This is the fail-closed inventory gate for tests that know which citizens a
/// build must contain. If link-time constructor collection drops a citizen row,
/// or if a wasm host exposes an empty inventory, the missing expected symbol is
/// reported before any shorter conformance sweep can pass.
pub fn run_registered_conformance_expecting(cx: &mut Cx, expected: &[&str]) -> Result<()> {
    let registry = CitizenRegistry::from_inventory()?;
    registry.ensure_contains_symbols(expected)?;
    cx.load_lib(&CitizenLib::all())?;
    run_conformance_rows(cx, registry.citizens())
}

/// Loads an explicit registry and runs each row's conformance fixture.
///
/// Use this DCE-safe path when a crate can name the citizen types it owns with
/// [`CitizenRegistry::register`]. The registry itself is loaded as a kernel
/// library before its fixture hooks run.
pub fn run_registry_conformance(cx: &mut Cx, registry: &CitizenRegistry) -> Result<()> {
    cx.load_lib(registry)?;
    run_conformance_rows(cx, registry.citizens())
}

/// Runs explicit-registry conformance after checking expected symbols.
///
/// This combines the DCE-safe registry path with the same completeness guard
/// used by [`run_registered_conformance_expecting`].
pub fn run_registry_conformance_expecting(
    cx: &mut Cx,
    registry: &CitizenRegistry,
    expected: &[&str],
) -> Result<()> {
    registry.ensure_contains_symbols(expected)?;
    run_registry_conformance(cx, registry)
}

fn run_conformance_rows<'a>(
    cx: &mut Cx,
    rows: impl IntoIterator<Item = &'a CitizenInfo>,
) -> Result<()> {
    for info in rows {
        (info.conformance)(cx)?;
    }
    Ok(())
}

/// Checks the citizen's default [`CitizenRuntime::example`] fixture round trip.
///
/// Convenience wrapper over [`check_fixture`] using the type's canonical
/// example value.
pub fn check_default_fixture<T>(cx: &mut Cx) -> Result<()>
where
    T: CitizenRuntime,
{
    check_fixture(cx, T::example())
}

/// Checks one runtime fixture's read-construct round trip and failure paths.
///
/// Encodes `fixture` to its constructor encoding, derives a deliberately wrong
/// version argument, and delegates to
/// [`check_value_fixture_with_wrong_version`] so the gate also rejects bad
/// versions alongside the round-trip, capability, and arity assertions.
pub fn check_fixture<T>(cx: &mut Cx, fixture: T) -> Result<()>
where
    T: CitizenRuntime,
{
    let original = cx.factory().opaque(Arc::new(fixture))?;
    let ObjectEncoding::Constructor { args, .. } = object_constructor_encoding(cx, &original)?
    else {
        unreachable!("object_constructor_encoding only returns constructor encodings");
    };
    let mut wrong_version = args.clone();
    if let Some(first) = wrong_version.first_mut() {
        *first = Expr::Symbol(Symbol::new("v999999"));
    }
    check_value_fixture_with_wrong_version(cx, original, Some(wrong_version))
}

/// Checks an already-built citizen [`Value`]'s read-construct round trip.
///
/// Like [`check_value_fixture_with_wrong_version`] but without a wrong-version
/// case, for citizens whose encoding has no version argument to corrupt.
pub fn check_value_fixture(cx: &mut Cx, original: Value) -> Result<()> {
    check_value_fixture_with_wrong_version(cx, original, None)
}

/// Asserts the full citizen read-construct contract for one value.
///
/// Confirms the constructor encoding renders to `#(<class> ...)` text, that
/// read-construct is denied without the capability and succeeds with it, that
/// the decoded value is `CitizenEq` to `original`, that a truncated argument
/// list is rejected, and, when `wrong_version` is supplied, that a mismatched
/// version is rejected. Read-construct stays capability-gated by the runtime
/// path; this helper only exercises the contract the kernel enforces.
pub fn check_value_fixture_with_wrong_version(
    cx: &mut Cx,
    original: Value,
    wrong_version: Option<Vec<Expr>>,
) -> Result<()> {
    let ObjectEncoding::Constructor { class, args } = object_constructor_encoding(cx, &original)?
    else {
        unreachable!("object_constructor_encoding only returns constructor encodings");
    };

    check_constructor_fixture(cx, original, class, args, wrong_version)
}

fn object_constructor_encoding(cx: &mut Cx, value: &Value) -> Result<ObjectEncoding> {
    let Some(encoder) = value.object().as_object_encoder() else {
        return Err(Error::Eval(
            "citizen conformance expects constructor encoding".to_owned(),
        ));
    };
    let encoding = encoder.object_encoding(cx)?;
    if !matches!(encoding, ObjectEncoding::Constructor { .. }) {
        return Err(Error::Eval(
            "citizen conformance expects constructor encoding".to_owned(),
        ));
    }
    Ok(encoding)
}

fn check_constructor_fixture(
    cx: &mut Cx,
    original: Value,
    class: Symbol,
    args: Vec<Expr>,
    wrong_version: Option<Vec<Expr>>,
) -> Result<()> {
    let text = render_constructor(&class, &args);
    let prefix = format!("#({class}");
    if !text.starts_with(&prefix) {
        return Err(Error::Eval(format!(
            "citizen constructor text {text:?} does not start with {prefix:?}"
        )));
    }

    let values = exprs_to_values(cx, &args)?;
    cx.with_capabilities(CapabilitySet::default(), |cx| {
        assert_capability_denied(cx.read_construct(&class, values.clone()))
    })?;

    let mut allowed = cx.capabilities().clone();
    allowed.insert(read_construct_capability());
    cx.with_capabilities(allowed, |cx| {
        let decoded = cx.read_construct(&class, values)?;
        if !values_citizen_eq(cx, &original, &decoded)? {
            return Err(Error::Eval(format!(
                "citizen {class} failed constructor round-trip equality"
            )));
        }

        let malformed = exprs_to_values(cx, &args[..args.len().saturating_sub(1)])?;
        if cx.read_construct(&class, malformed).is_ok() {
            return Err(Error::Eval(format!(
                "citizen {class} accepted malformed arity"
            )));
        }

        if let Some(wrong_version) = wrong_version {
            let wrong_version = exprs_to_values(cx, &wrong_version)?;
            if cx.read_construct(&class, wrong_version).is_ok() {
                return Err(Error::Eval(format!(
                    "citizen {class} accepted wrong version"
                )));
            }
        }

        Ok(())
    })?;

    Ok(())
}

fn exprs_to_values(cx: &mut Cx, exprs: &[Expr]) -> Result<Vec<Value>> {
    exprs.iter().map(|expr| value_from_expr(cx, expr)).collect()
}

fn assert_capability_denied(result: Result<Value>) -> Result<()> {
    match result {
        Err(Error::CapabilityDenied { capability })
            if capability == read_construct_capability() =>
        {
            Ok(())
        }
        Err(err) => Err(field_error(
            "read-construct",
            format!("expected capability denial, found {err}"),
        )),
        Ok(_) => Err(field_error(
            "read-construct",
            "expected capability denial, found success",
        )),
    }
}

fn render_constructor(class: &Symbol, args: &[Expr]) -> String {
    let mut out = format!("#({class}");
    for arg in args {
        out.push(' ');
        out.push_str(&render_expr(arg));
    }
    out.push(')');
    out
}

fn render_expr(expr: &Expr) -> String {
    match expr {
        Expr::Nil => "nil".to_owned(),
        Expr::Bool(value) => value.to_string(),
        Expr::Number(value) => value.canonical.clone(),
        Expr::Symbol(value) => value.to_string(),
        Expr::String(value) => format!("{value:?}"),
        Expr::List(items) => {
            let rendered = items.iter().map(render_expr).collect::<Vec<_>>().join(" ");
            format!("({rendered})")
        }
        other => format!("{other:?}"),
    }
}
