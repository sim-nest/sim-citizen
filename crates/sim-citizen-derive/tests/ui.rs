#[test]
fn ui_non_citizen_attribute_contract() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass/non_citizen_valid.rs");
    t.compile_fail("tests/ui/fail/non_citizen_missing_kind.rs");
    t.compile_fail("tests/ui/fail/non_citizen_missing_descriptor.rs");
    t.compile_fail("tests/ui/fail/non_citizen_duplicate_reason.rs");
    t.compile_fail("tests/ui/fail/non_citizen_empty_reason.rs");
    t.compile_fail("tests/ui/fail/non_citizen_unsupported_key.rs");
}
