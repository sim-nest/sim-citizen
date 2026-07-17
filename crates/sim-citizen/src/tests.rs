use sim_kernel::{Cx, DefaultFactory, Error, Expr, NoopEvalPolicy, ObjectEncode, Symbol};

use crate::{
    CitizenField, CitizenLib, citizen_census_markdown, example::Point, non_citizen_card,
    non_citizen_census_markdown, registered_citizens, registered_non_citizens,
    run_registered_conformance, value_from_expr, value_to_expr,
};

#[sim_citizen_derive::non_citizen(
    reason = "runtime-owned state",
    kind = "live-handle",
    descriptor = "example/live-handle"
)]
struct ExampleLiveHandle;

#[test]
fn point_is_registered_by_inventory() {
    let point = registered_citizens()
        .find(|info| info.symbol == "example/Point")
        .expect("point citizen should be registered");
    assert_eq!(point.version, 1);
    assert_eq!(point.arity, 2);
    assert_eq!(point.crate_name, "sim-citizen");
}

#[test]
fn point_round_trips_through_conformance() {
    let mut cx = cx();
    run_registered_conformance(&mut cx).unwrap();
}

#[test]
fn non_citizen_exemption_is_registered_by_inventory() {
    let _ = ExampleLiveHandle;
    let info = registered_non_citizens()
        .find(|info| info.type_name == "ExampleLiveHandle")
        .expect("non-citizen exemption should be registered");
    assert_eq!(info.crate_name, "sim-citizen");
    assert_eq!(info.reason, "runtime-owned state");
    assert_eq!(info.kind, "live-handle");
    assert_eq!(info.descriptor, "example/live-handle");
}

#[test]
fn point_malformed_arity_returns_error() {
    let mut cx = cx();
    cx.load_lib(&CitizenLib::all()).unwrap();
    cx.grant(sim_kernel::read_construct_capability());

    let value = value_from_expr(&mut cx, &Expr::Symbol(Symbol::new("v1"))).unwrap();
    let err = cx
        .read_construct(&Symbol::qualified("example", "Point"), vec![value])
        .expect_err("malformed arity must fail");
    assert!(matches!(err, Error::Eval(message) if message.contains("expects 3")));
}

#[test]
fn point_wrong_capability_fails_closed() {
    let mut cx = cx();
    cx.load_lib(&CitizenLib::all()).unwrap();
    let point = Point { x: 1, y: 2 };
    let encoding = point.object_encoding(&mut cx).unwrap();
    let sim_kernel::ObjectEncoding::Constructor { class, args } = encoding else {
        panic!("point should use constructor encoding");
    };
    let values = args
        .iter()
        .map(|arg| value_from_expr(&mut cx, arg))
        .collect::<sim_kernel::Result<Vec<_>>>()
        .unwrap();

    let err = cx
        .read_construct(&class, values)
        .expect_err("read-construct must be capability-gated");
    assert!(
        matches!(err, Error::CapabilityDenied { capability } if capability == sim_kernel::read_construct_capability())
    );
}

#[test]
fn field_helpers_decode_scalar_list_and_option() {
    let mut cx = cx();
    let expr = vec![1_i64, 2, 3].encode_field();
    let value = value_from_expr(&mut cx, &expr).unwrap();
    let decoded = Vec::<i64>::decode_field_value(&mut cx, value, "numbers").unwrap();
    assert_eq!(decoded, vec![1, 2, 3]);

    let expr = Option::<String>::None.encode_field();
    let value = value_from_expr(&mut cx, &expr).unwrap();
    let decoded = Option::<String>::decode_field_value(&mut cx, value, "maybe").unwrap();
    assert_eq!(decoded, None);
}

#[test]
fn substrate_citizen_census_contains_point() {
    let generated = citizen_census_markdown();
    assert!(generated.contains("| `example/Point` | 1 | 2 | `sim-citizen` |"));
    assert!(generated.contains("# Generated Non-Citizen Exemption Census"));
    assert!(generated.contains(
        "| `ExampleLiveHandle` | `live-handle` | `example/live-handle` | `sim-citizen` | runtime-owned state |"
    ));
}

#[test]
fn substrate_non_citizen_census_contains_live_handle() {
    let generated = non_citizen_census_markdown();
    assert!(generated.contains(
        "| `ExampleLiveHandle` | `live-handle` | `example/live-handle` | `sim-citizen` | runtime-owned state |"
    ));
}

#[test]
fn non_citizen_card_renders_registered_exemption_fields() {
    let info = registered_non_citizens()
        .find(|info| info.type_name == "ExampleLiveHandle")
        .expect("non-citizen exemption should be registered");
    let mut cx = cx();
    let card = non_citizen_card(&mut cx, info).unwrap();
    let expr = value_to_expr(&mut cx, card, "card").unwrap();
    let Expr::Map(entries) = expr else {
        panic!("non-citizen card should project to a map expression");
    };
    assert_eq!(
        map_string_field(&entries, "type_name"),
        Some("ExampleLiveHandle")
    );
    assert_eq!(map_string_field(&entries, "crate"), Some("sim-citizen"));
    assert_eq!(
        map_string_field(&entries, "reason"),
        Some("runtime-owned state")
    );
    assert_eq!(map_string_field(&entries, "kind"), Some("live-handle"));
    assert_eq!(
        map_string_field(&entries, "descriptor"),
        Some("example/live-handle")
    );
}

fn cx() -> Cx {
    Cx::new(
        std::sync::Arc::new(NoopEvalPolicy),
        std::sync::Arc::new(DefaultFactory),
    )
}

fn map_string_field<'a>(entries: &'a [(Expr, Expr)], field: &str) -> Option<&'a str> {
    entries.iter().find_map(|(key, value)| match (key, value) {
        (Expr::Symbol(symbol), Expr::String(value)) if symbol.name.as_ref() == field => {
            Some(value.as_str())
        }
        _ => None,
    })
}
