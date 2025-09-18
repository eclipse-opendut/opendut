use viper_pygen::pygen;

struct Fubar;

#[pygen]
impl Fubar {

    #[allow(unused_variables)]
    #[cfg_attr(test, allow(unused_variables))]
    fn foo() {}

    #[cfg_attr(test, allow(unused_variables))]
    fn bar() {}
}

fn main() {
    let expected = indoc::indoc!("
        class Fubar:
            def foo():
                pass
            def bar():
                pass
    ");

    assert_eq!(Fubar::GENERATED_PYTHON_CODE, expected);
}
