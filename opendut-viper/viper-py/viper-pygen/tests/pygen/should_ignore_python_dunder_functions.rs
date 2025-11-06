#![allow(unused_variables)]

use opendut_viper_pygen::pygen;

struct Fubar;

#[pygen]
impl Fubar {

    fn inc(value: i32) -> i32 { value + 1 }

    fn __foo(this: &Self) {}

    fn __bar() {}
}

fn main() {
    let expected = indoc::indoc!("
        class Fubar:
            def inc(value: int) -> int:
                pass
    ");

    assert_eq!(Fubar::GENERATED_PYTHON_CODE, expected);
}
