use viper_pygen::pygen;

struct Fubar;

#[pygen]
impl Fubar {

    const FOO: i32 = 42;

    fn bla() {}
}

fn main() {
    let expected = indoc::indoc!("
        class Fubar:
            def bla():
                pass
    ");

    assert_eq!(Fubar::GENERATED_PYTHON_CODE, expected);
}
