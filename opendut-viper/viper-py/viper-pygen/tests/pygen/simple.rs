#![allow(unused_variables)]

use viper_pygen::pygen;

struct TestCase;

#[pygen]
impl TestCase {
    fn assert_equals(a: i32, b: i32, message: Option<String>) {}
}

fn main() {
    let expected = indoc::indoc!("
        class TestCase:
            def assert_equals(a: int, b: int, message: str):
                pass
    ");

    assert_eq!(TestCase::GENERATED_PYTHON_CODE, expected);
}
