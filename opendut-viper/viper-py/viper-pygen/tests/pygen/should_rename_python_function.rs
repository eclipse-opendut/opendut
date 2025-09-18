use viper_pygen::pygen;

struct Elmar;

#[pygen]
impl Elmar {
    #[viper(name = "vivian")]
    fn jessica() {}

    #[viper(name = "y")]
    fn x() {}
}

fn main() {
    let expected = indoc::indoc!("
        class Elmar:
            def vivian():
                pass
            def y():
                pass
    ");

    assert_eq!(Elmar::GENERATED_PYTHON_CODE, expected);
}
