use viper_pygen::pygen;

struct Fubar;

#[pygen]
#[allow(unused_variables)]
#[cfg_attr(test, allow(unused_variables))]
impl Fubar {

    fn bla(x: i32, #[viper(skip)] vskgsgm: i32) {}

    fn foo(x: i32, #[viper(skip)] _vm: String) {}
}

fn main() {
    let expected = indoc::indoc!("
        class Fubar:
            def bla(x: int):
                pass
            def foo(x: int):
                pass
    ");

    assert_eq!(Fubar::GENERATED_PYTHON_CODE, expected);
}
