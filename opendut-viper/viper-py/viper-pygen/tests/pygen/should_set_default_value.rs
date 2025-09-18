use viper_pygen::pygen;

struct Struct;

#[pygen]
#[allow(unused_variables)]
#[cfg_attr(test, allow(unused_variables))]
impl Struct {
    fn fubar(#[viper(default = "Default Value")] message: String) {}
}

fn main() {
    let expected = indoc::indoc!("
        class Struct:
            def fubar(message: str = \"Default Value\"):
                pass
    ");

    assert_eq!(Struct::GENERATED_PYTHON_CODE, expected);
}
