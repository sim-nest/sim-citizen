use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use sim_kernel::{
    Cx, DefaultFactory, Error, Expr, MatchScore, NoopEvalPolicy, NumberLiteral, ObjectEncode,
    Shape, ShapeDoc, ShapeMatch, Symbol, Value,
    card::{Card, card_fixed_predicates},
};

use crate::{
    CitizenField, CitizenLib, CitizenRegistry, citizen_card, citizen_census_markdown,
    citizen_registry_census_markdown, example::Point, expr_citizen_eq, non_citizen_card,
    non_citizen_census_markdown, registered_citizens, registered_non_citizens,
    run_registered_conformance, run_registered_conformance_expecting, run_registry_conformance,
    run_registry_conformance_expecting, value_from_expr, value_to_expr,
};

#[derive(Clone, Debug, Default, PartialEq, sim_citizen_derive::Citizen)]
#[citizen(symbol = "example/Float", version = 1)]
struct ExampleFloat {
    value: f64,
}

#[derive(Clone, Debug, PartialEq, sim_citizen_derive::Citizen)]
#[citizen(
    symbol = "example/FixtureCounter",
    version = 1,
    example = fixture_counter_example,
    fixtures = fixture_counter_fixtures
)]
struct FixtureCounter {
    value: i64,
}

#[sim_citizen_derive::non_citizen(
    reason = "runtime-owned state",
    kind = "live-handle",
    descriptor = "example/live-handle"
)]
struct ExampleLiveHandle;

static FIXTURE_COUNTER_EXAMPLE_CALLS: AtomicUsize = AtomicUsize::new(0);
static FIXTURE_COUNTER_FIXTURE_FACTORY_CALLS: AtomicUsize = AtomicUsize::new(0);
static FIXTURE_COUNTER_FIXTURE_EMISSIONS: AtomicUsize = AtomicUsize::new(0);

fn fixture_counter_example() -> FixtureCounter {
    FIXTURE_COUNTER_EXAMPLE_CALLS.fetch_add(1, Ordering::Relaxed);
    FixtureCounter { value: 7 }
}

fn fixture_counter_fixtures() -> FixtureCounterFixtures {
    FIXTURE_COUNTER_FIXTURE_FACTORY_CALLS.fetch_add(1, Ordering::Relaxed);
    FixtureCounterFixtures
}

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
fn inventory_conformance_can_require_expected_symbols() {
    let mut cx = cx();
    run_registered_conformance_expecting(
        &mut cx,
        &["example/Point", "example/Float", "example/FixtureCounter"],
    )
    .unwrap();
}

#[test]
fn inventory_conformance_fails_when_expected_symbol_is_absent() {
    let mut cx = cx();
    let err = run_registered_conformance_expecting(&mut cx, &["example/Missing"])
        .expect_err("missing expected citizen must fail closed");
    assert!(matches!(
        err,
        Error::HostError(message)
            if message.contains("citizen registry incomplete")
                && message.contains("example/Missing")
    ));
}

#[test]
fn explicit_registry_loads_checks_and_renders_without_inventory_lookup() {
    let mut registry = CitizenRegistry::new();
    registry.register::<Point>().unwrap();
    assert_eq!(registry.len(), 1);
    assert!(!registry.is_empty());
    assert_eq!(
        registry.missing_symbols(&["example/Point"]),
        Vec::<&str>::new()
    );

    let mut cx = cx();
    run_registry_conformance_expecting(&mut cx, &registry, &["example/Point"]).unwrap();
    assert!(
        cx.registry()
            .class_by_symbol(&Symbol::qualified("example", "Point"))
            .is_some()
    );

    let generated = citizen_registry_census_markdown(&registry);
    assert!(generated.contains("Total citizens: 1"));
    assert!(generated.contains("| `example/Point` | 1 | 2 | `sim-citizen` |"));
}

#[test]
fn explicit_registry_rejects_duplicate_symbols() {
    let mut registry = CitizenRegistry::new();
    registry.register::<Point>().unwrap();
    let err = registry
        .register::<Point>()
        .err()
        .expect("duplicate explicit citizen must fail closed");
    assert!(matches!(
        err,
        Error::Eval(message)
            if message.contains("duplicate citizen registration for example/Point")
    ));
}

#[test]
fn explicit_registry_conformance_fails_when_expected_symbol_is_absent() {
    let mut registry = CitizenRegistry::new();
    registry.register::<Point>().unwrap();

    let mut cx = cx();
    let err = run_registry_conformance_expecting(&mut cx, &registry, &["example/Missing"])
        .expect_err("missing explicit citizen must fail closed");
    assert!(matches!(
        err,
        Error::HostError(message)
            if message.contains("citizen registry incomplete")
                && message.contains("example/Missing")
    ));
}

#[test]
fn explicit_registry_conformance_runs_without_expected_list() {
    let mut registry = CitizenRegistry::new();
    registry.register::<Point>().unwrap();

    let mut cx = cx();
    run_registry_conformance(&mut cx, &registry).unwrap();
}

#[test]
fn custom_example_and_fixtures_paths_drive_conformance() {
    let before_example = FIXTURE_COUNTER_EXAMPLE_CALLS.load(Ordering::Relaxed);
    let before_fixtures = FIXTURE_COUNTER_FIXTURE_FACTORY_CALLS.load(Ordering::Relaxed);
    let before_emissions = FIXTURE_COUNTER_FIXTURE_EMISSIONS.load(Ordering::Relaxed);

    let mut cx = cx();
    cx.load_lib(&CitizenLib::all()).unwrap();
    let info = registered_citizens()
        .find(|info| info.symbol == "example/FixtureCounter")
        .expect("custom fixture citizen should be registered");

    (info.conformance)(&mut cx).unwrap();

    assert!(FIXTURE_COUNTER_EXAMPLE_CALLS.load(Ordering::Relaxed) > before_example);
    assert!(FIXTURE_COUNTER_FIXTURE_FACTORY_CALLS.load(Ordering::Relaxed) > before_fixtures);
    assert!(FIXTURE_COUNTER_FIXTURE_EMISSIONS.load(Ordering::Relaxed) >= before_emissions + 2);
}

#[test]
fn point_class_members_publish_version_arity_and_named_fields() {
    let mut cx = cx();
    cx.load_lib(&CitizenLib::all()).unwrap();
    let class = point_class(&cx);
    let members = class.object().as_class().unwrap().members(&mut cx).unwrap();
    let expr = value_to_expr(&mut cx, members, "members").unwrap();
    let Expr::Map(entries) = expr else {
        panic!("class members should project to a map expression");
    };

    assert!(matches!(
        map_field(&entries, "version"),
        Some(Expr::Number(NumberLiteral { domain, canonical }))
            if *domain == Symbol::qualified("citizen", "int") && canonical == "1"
    ));
    assert!(matches!(
        map_field(&entries, "arity"),
        Some(Expr::Number(NumberLiteral { domain, canonical }))
            if *domain == Symbol::qualified("citizen", "int") && canonical == "2"
    ));
    assert_eq!(
        map_field(&entries, "fields"),
        Some(&Expr::List(vec![
            Expr::Symbol(Symbol::new("x")),
            Expr::Symbol(Symbol::new("y")),
        ]))
    );
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
fn point_shape_hooks_fall_back_to_nil_without_core_any_shape() {
    let mut cx = cx();
    cx.load_lib(&CitizenLib::all()).unwrap();
    let class = point_class(&cx);
    let class_view = class.object().as_class().unwrap();
    let read_constructor = class_view.read_constructor(&mut cx).unwrap().unwrap();
    let read_constructor = read_constructor.object().as_read_constructor().unwrap();

    for shape in [
        class_view.constructor_shape(&mut cx).unwrap(),
        class_view.instance_shape(&mut cx).unwrap(),
        read_constructor.args_shape(&mut cx).unwrap(),
    ] {
        let expr = value_to_expr(&mut cx, shape, "shape").unwrap();
        assert_eq!(expr, Expr::Nil);
    }
}

#[test]
fn point_shape_hooks_publish_core_any_when_registered() {
    let mut cx = cx();
    cx.load_lib(&CitizenLib::all()).unwrap();
    register_core_any_shape(&mut cx);
    let class = point_class(&cx);
    let class_view = class.object().as_class().unwrap();
    let read_constructor = class_view.read_constructor(&mut cx).unwrap().unwrap();
    let read_constructor = read_constructor.object().as_read_constructor().unwrap();

    for shape in [
        class_view.constructor_shape(&mut cx).unwrap(),
        class_view.instance_shape(&mut cx).unwrap(),
        read_constructor.args_shape(&mut cx).unwrap(),
    ] {
        assert_eq!(
            shape.object().as_shape().and_then(Shape::symbol),
            Some(Symbol::qualified("core", "Any"))
        );
    }
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
fn wrong_number_domains_fail_integer_read_construct() {
    let mut cx = cx();
    cx.load_lib(&CitizenLib::all()).unwrap();
    cx.grant(sim_kernel::read_construct_capability());
    let version = value_from_expr(&mut cx, &Expr::Symbol(Symbol::new("v1"))).unwrap();
    let wrong_x = value_from_expr(
        &mut cx,
        &Expr::Number(NumberLiteral {
            domain: Symbol::qualified("numbers", "f64"),
            canonical: "1".to_owned(),
        }),
    )
    .unwrap();
    let y = value_from_expr(&mut cx, &2_i64.encode_field()).unwrap();

    let err = cx
        .read_construct(
            &Symbol::qualified("example", "Point"),
            vec![version, wrong_x, y],
        )
        .expect_err("wrong integer domain must fail read-construct");
    assert!(matches!(
        err,
        Error::Eval(message)
            if message.contains("expected number domain citizen/int, found numbers/f64")
    ));
}

#[test]
fn wrong_number_domains_fail_f64_read_construct() {
    let mut cx = cx();
    cx.load_lib(&CitizenLib::all()).unwrap();
    cx.grant(sim_kernel::read_construct_capability());
    let version = value_from_expr(&mut cx, &Expr::Symbol(Symbol::new("v1"))).unwrap();
    let wrong_value = value_from_expr(&mut cx, &1_i64.encode_field()).unwrap();

    let err = cx
        .read_construct(
            &Symbol::qualified("example", "Float"),
            vec![version, wrong_value],
        )
        .expect_err("wrong f64 domain must fail read-construct");
    assert!(matches!(
        err,
        Error::Eval(message)
            if message.contains("expected number domain numbers/f64, found citizen/int")
    ));
}

#[test]
fn wrong_number_domains_are_not_citizen_equal() {
    let f64_expr = Expr::Number(NumberLiteral {
        domain: Symbol::qualified("numbers", "f64"),
        canonical: "1".to_owned(),
    });
    let int_expr = Expr::Number(NumberLiteral {
        domain: Symbol::qualified("citizen", "int"),
        canonical: "1".to_owned(),
    });
    let other_f64_name = Expr::Number(NumberLiteral {
        domain: Symbol::qualified("example", "f64"),
        canonical: "1".to_owned(),
    });

    assert!(!expr_citizen_eq(&f64_expr, &int_expr));
    assert!(!expr_citizen_eq(&int_expr, &f64_expr));
    assert!(!expr_citizen_eq(&f64_expr, &other_f64_name));
    assert!(!expr_citizen_eq(&other_f64_name, &f64_expr));
}

#[test]
fn f64_special_values_round_trip_through_field_codec() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let encoded = value.encode_field();
        let decoded = f64::decode_field_expr(&encoded, "value").unwrap();
        let reencoded = decoded.encode_field();
        assert!(expr_citizen_eq(&encoded, &reencoded));
        if value.is_nan() {
            assert!(decoded.is_nan());
        } else {
            assert_eq!(decoded, value);
        }
    }
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
fn citizen_card_uses_kernel_card_schema_and_subject() {
    let mut cx = cx();
    cx.load_lib(&CitizenLib::all()).unwrap();
    let info = registered_citizens()
        .find(|info| info.symbol == "example/Point")
        .expect("point citizen should be registered");

    let card = citizen_card(&mut cx, info).unwrap();
    assert!(card.object().downcast_ref::<Card>().is_some());

    let expr = value_to_expr(&mut cx, card, "card").unwrap();
    let Expr::Map(entries) = expr else {
        panic!("citizen card should project to a map expression");
    };
    let keys = entries
        .iter()
        .map(|(key, _)| match key {
            Expr::Symbol(symbol) => symbol.clone(),
            other => panic!("card keys must be symbols, found {other:?}"),
        })
        .take(card_fixed_predicates().len())
        .collect::<Vec<_>>();
    let projected_fixed_fields = card_fixed_predicates()
        .into_iter()
        .map(|symbol| Symbol::new(symbol.name.to_string()))
        .collect::<Vec<_>>();
    assert_eq!(keys, projected_fixed_fields);
    assert_eq!(
        map_field(&entries, "subject"),
        Some(&Expr::Symbol(Symbol::qualified("example", "Point")))
    );
    assert_eq!(
        map_field(&entries, "kind"),
        Some(&Expr::Symbol(Symbol::qualified("core", "class")))
    );
    assert_eq!(
        map_field(&entries, "args"),
        Some(&Expr::Symbol(Symbol::qualified("core", "Any")))
    );
    assert_eq!(
        map_field(&entries, "result"),
        Some(&Expr::Symbol(Symbol::qualified("core", "Any")))
    );
    assert_eq!(map_field(&entries, "shape-known"), Some(&Expr::Bool(false)));
    assert_eq!(
        map_field(&entries, "crate"),
        Some(&Expr::String("sim-citizen".to_owned()))
    );
    assert!(matches!(
        map_field(&entries, "version"),
        Some(Expr::Number(NumberLiteral { domain, canonical }))
            if *domain == Symbol::qualified("citizen", "int") && canonical == "1"
    ));
    assert!(matches!(
        map_field(&entries, "arity"),
        Some(Expr::Number(NumberLiteral { domain, canonical }))
            if *domain == Symbol::qualified("citizen", "int") && canonical == "2"
    ));
    assert_eq!(
        map_field(&entries, "fields"),
        Some(&Expr::List(vec![
            Expr::Symbol(Symbol::new("x")),
            Expr::Symbol(Symbol::new("y")),
        ]))
    );
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

fn point_class(cx: &Cx) -> Value {
    cx.registry()
        .class_by_symbol(&Symbol::qualified("example", "Point"))
        .cloned()
        .expect("point citizen class should be loaded")
}

fn register_core_any_shape(cx: &mut Cx) {
    let shape = cx.factory().opaque(Arc::new(TestAnyShape)).unwrap();
    cx.registry_mut()
        .register_shape_value(Symbol::qualified("core", "Any"), shape)
        .unwrap();
}

#[derive(Debug)]
struct TestAnyShape;

struct FixtureCounterFixtures;

impl IntoIterator for FixtureCounterFixtures {
    type Item = FixtureCounter;
    type IntoIter = FixtureCounterIter;

    fn into_iter(self) -> Self::IntoIter {
        FixtureCounterIter { index: 0 }
    }
}

struct FixtureCounterIter {
    index: usize,
}

impl Iterator for FixtureCounterIter {
    type Item = FixtureCounter;

    fn next(&mut self) -> Option<Self::Item> {
        let value = match self.index {
            0 => -1,
            1 => 42,
            _ => return None,
        };
        self.index += 1;
        FIXTURE_COUNTER_FIXTURE_EMISSIONS.fetch_add(1, Ordering::Relaxed);
        Some(FixtureCounter { value })
    }
}

impl Shape for TestAnyShape {
    fn symbol(&self) -> Option<Symbol> {
        Some(Symbol::qualified("core", "Any"))
    }

    fn is_total(&self) -> bool {
        true
    }

    fn check_value(&self, _cx: &mut Cx, _value: Value) -> sim_kernel::Result<ShapeMatch> {
        Ok(ShapeMatch::accept(MatchScore::exact(1)))
    }

    fn check_expr(&self, _cx: &mut Cx, _expr: &Expr) -> sim_kernel::Result<ShapeMatch> {
        Ok(ShapeMatch::accept(MatchScore::exact(1)))
    }

    fn describe(&self, _cx: &mut Cx) -> sim_kernel::Result<ShapeDoc> {
        Ok(ShapeDoc::new("any"))
    }
}

fn map_field<'a>(entries: &'a [(Expr, Expr)], field: &str) -> Option<&'a Expr> {
    entries.iter().find_map(|(key, value)| match key {
        Expr::Symbol(symbol) if symbol.name.as_ref() == field => Some(value),
        _ => None,
    })
}

fn map_string_field<'a>(entries: &'a [(Expr, Expr)], field: &str) -> Option<&'a str> {
    match map_field(entries, field) {
        Some(Expr::String(value)) => Some(value.as_str()),
        _ => None,
    }
}
