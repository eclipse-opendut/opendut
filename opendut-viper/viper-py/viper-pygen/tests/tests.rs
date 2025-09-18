#[test]
fn test() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/pygen/should_fail_on_enums.rs");
    tests.pass("tests/pygen/should_generate_doc.rs");
    tests.pass("tests/pygen/should_generate_self_function_argument_from_this.rs");
    tests.pass("tests/pygen/should_generate_type_hint.rs");
    tests.pass("tests/pygen/should_ignore_python_dunder_functions.rs");
    tests.pass("tests/pygen/should_keep_original_impl.rs");
    tests.pass("tests/pygen/should_not_fail.rs");
    tests.pass("tests/pygen/should_parse_function_attributes.rs");
    tests.pass("tests/pygen/should_parse_function_return_types.rs");
    tests.pass("tests/pygen/should_parse_impl_associated_constants.rs");
    tests.pass("tests/pygen/should_parse_impl_attributes.rs");
    tests.pass("tests/pygen/should_rename_python_function.rs");
    tests.pass("tests/pygen/should_set_default_value.rs");
    tests.pass("tests/pygen/should_skip_argument.rs");
    tests.pass("tests/pygen/simple.rs");
}
