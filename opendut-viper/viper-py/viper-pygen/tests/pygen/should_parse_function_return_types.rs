use opendut_viper_pygen::pygen;

struct Fubar;

#[pygen]
impl Fubar {

    fn foo() -> i32 {
        todo!()
    }

    fn bar() -> Result<(), String> {
        todo!()
    }

    fn test() -> Option<String> {
        todo!()
    }
}

fn main() {
    let expected = indoc::indoc!("
        class Fubar:
            def foo() -> int:
                pass
            def bar() -> str:
                pass
            def test() -> str:
                pass
    ");

    assert_eq!(Fubar::GENERATED_PYTHON_CODE, expected);
}
