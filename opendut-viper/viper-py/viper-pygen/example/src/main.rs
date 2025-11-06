use opendut_viper_pygen::pygen;

struct Test {}

#[pygen]
#[allow(dead_code)]
#[doc = "Python code will be generated here!"]
/// This is my awesome test!
impl Test {

    #[allow(unused_variables)]
    #[viper(name="z")]
    fn bla(#[viper(skip)] x: i32, y: i32) -> i32 {
        x + 1
    }
}

fn main() {
    println!("Python:");
    println!("========================================");
    print!("{}", Test::GENERATED_PYTHON_CODE);
    println!("========================================");
    assert_eq!(Test::bla(1, 2), 2);

    let expected = indoc::indoc!("
        class Test:
            def z(y):
                pass
    ");
    assert_eq!(Test::GENERATED_PYTHON_CODE, expected);
}
