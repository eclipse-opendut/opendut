use rustpython_vm::*;

#[pymodule]
pub mod container {
    use viper_pygen::pygen;
    use rustpython_vm::*;

    #[pyattr]
    #[pyclass(name = "ContainerRuntimeProxy")]
    #[derive(PyPayload, Debug)]
    pub struct Fubar {
        name: String
    }

    #[pyclass]
    #[pygen]
    impl Fubar {
        pub fn new(name: String) -> PyResult<Self> {
            Ok(Self { name })
        }

        #[pymethod]
        fn bar() -> PyResult<()> {
            todo!()
        }
        
    }
}

fn main() {
    let expected = indoc::indoc!("
        class Fubar:
            def new(name: str):
                pass
            def bar():
                pass
    ");

    assert_eq!(container::Fubar::GENERATED_PYTHON_CODE, expected);
}
