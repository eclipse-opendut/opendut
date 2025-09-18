use viper_pygen::pygen;

struct Fubar;

#[pygen]
impl Fubar {

    #[allow(unused_variables)]
    fn foo(a: Vec<String>) {}

    #[allow(unused_variables)]
    fn test(&self) {}

    #[allow(unused_variables)]
    fn test2(b: i32, c: f64, d: String, e: bool, f: Option<String>) {}
}

fn main() {
    let expected = indoc::indoc!("
        class Fubar:
            def foo(a: list):
                pass
            def test(self):
                pass
            def test2(b: int, c: float, d: str, e: bool, f: str):
                pass
    ");

    assert_eq!(Fubar::GENERATED_PYTHON_CODE, expected);
}
