use opendut_viper_pygen::pygen;

struct Fubar;

#[pygen]
#[allow(unused_variables)]
#[cfg_attr(test, allow(unused_variables))]
impl Fubar {

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
