#[test]
fn ui_citizen_attribute_contract() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass/citizen_custom_fixtures.rs");
    t.pass("tests/ui/pass/citizen_helper_name_collision.rs");
    t.compile_fail("tests/ui/fail/citizen_duplicate_symbol.rs");
    t.compile_fail("tests/ui/fail/citizen_field_citizen_mode.rs");
    t.compile_fail("tests/ui/fail/citizen_field_duplicate_with.rs");
    t.compile_fail("tests/ui/fail/citizen_field_list_mode.rs");
    t.compile_fail("tests/ui/fail/citizen_field_unsupported_key.rs");
    t.compile_fail("tests/ui/fail/citizen_missing_default.rs");
    t.compile_fail("tests/ui/fail/citizen_unsupported_key.rs");
    t.compile_fail("tests/ui/fail/citizen_with_bad_decode.rs");
    t.compile_fail("tests/ui/fail/citizen_with_bad_encode.rs");
}

#[test]
fn ui_non_citizen_attribute_contract() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass/non_citizen_missing_descriptor.rs");
    t.pass("tests/ui/pass/non_citizen_valid.rs");
    t.compile_fail("tests/ui/fail/non_citizen_missing_kind.rs");
    t.compile_fail("tests/ui/fail/non_citizen_duplicate_reason.rs");
    t.compile_fail("tests/ui/fail/non_citizen_empty_reason.rs");
    t.compile_fail("tests/ui/fail/non_citizen_unsupported_key.rs");
}
