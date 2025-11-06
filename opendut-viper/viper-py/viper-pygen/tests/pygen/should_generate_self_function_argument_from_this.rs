#![allow(unused_variables)]

use opendut_viper_pygen::pygen;

struct Fubar;

#[pygen]
impl Fubar {

    fn foo(this: &Self) {}

    fn bar(_this: &Self) {}
}

fn main() {
    let expected = indoc::indoc!("
        class Fubar:
            def foo(self):
                pass
            def bar(self):
                pass
    ");

    assert_eq!(Fubar::GENERATED_PYTHON_CODE, expected);
}
