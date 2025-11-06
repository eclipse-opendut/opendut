use opendut_viper_pygen::pygen;

struct Fubar;

#[pygen]
/// This is my awesome struct!
/// Second Line
impl Fubar {

    #[doc = "Function Documentation"]
    fn a() {}

    /// This is my awesome function!
    /// Second Line
    fn b() {}

    fn c() {}
}


struct Foo;

#[pygen]
/// This struct documentation is one line long
impl Foo {

    /// This function documentation is also one line long
    fn d() {}
}

fn main() {
    let expected_fubar = indoc::indoc!(r#"
        class Fubar:
            """
            This is my awesome struct!
            Second Line
            """
            def a():
                """Function Documentation"""
                pass
            def b():
                """
                This is my awesome function!
                Second Line
                """
                pass
            def c():
                pass
    "#);

    let expected_foo = indoc::indoc!(r#"
        class Foo:
            """This struct documentation is one line long"""
            def d():
                """This function documentation is also one line long"""
                pass
    "#);

    assert_eq!(Fubar::GENERATED_PYTHON_CODE, expected_fubar);
    assert_eq!(Foo::GENERATED_PYTHON_CODE, expected_foo);
}
