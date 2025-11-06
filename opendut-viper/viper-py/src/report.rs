#![allow(clippy::module_inception)]
use rustpython_vm::{pymodule, PyResult};

pub trait ReportPropertiesCollector {
    fn set_number_property(&self, name: String, value: i64) -> PyResult<()>;
    fn set_string_property(&self, name: String, value: String) -> PyResult<()>;
    fn set_file_property(&self, value: String) -> PyResult<()>;
}

#[pymodule]
pub mod report {
    use rustpython_vm::builtins::{PyBaseExceptionRef, PyInt, PyStr};
    use rustpython_vm::function::{KwArgs, PosArgs};
    use rustpython_vm::{pyclass, PyObjectRef, PyPayload, PyResult, VirtualMachine};
    use std::fmt::Debug;
    use crate::report::ReportPropertiesCollector;

    #[pyclass(name = "ReportProperties", no_attr)]
    #[derive(PyPayload)]
    pub struct PyReportProperties {
        collector: Box<dyn ReportPropertiesCollector>,
    }

    #[pyclass]
    #[opendut_viper_pygen::pygen]
    impl PyReportProperties {

        #[viper(skip)]
        pub fn new(collector: Box<dyn ReportPropertiesCollector>) -> PyReportProperties {
            Self { collector }
        }

        /// Adds a property with the given `name` and `value` to the report.
        ///
        /// # Example
        ///
        /// ```python
        /// self.report.property("result", 42)
        /// self.report.property("rating", "Looks good to me")
        /// ```
        #[pymethod]
        fn property(&self, name: String, value: PyObjectRef, #[viper(skip)] vm: &VirtualMachine) -> PyResult<()> {
            if let Some(value) = value.downcast_ref::<PyStr>() {
                self.collector.set_string_property(name, value.to_string())
            }
            else if let Some(value) = value.downcast_ref::<PyInt>() {
                let value = value.as_bigint();
                let value = i64::try_from(value)
                    .expect("downcast from `BigInt` to `i64`");
                self.collector.set_number_property(name, value)
            }
            else {
                Err(unsupported_property_value_type(name, vm))
            }
        }

        /// Adds the properties provided as keyworded args to the report.
        ///
        /// # Example
        ///
        /// ```python
        /// self.report.properties(
        ///     result=42,
        ///     rating="Looks good to me",
        /// )
        /// ```
        #[pymethod]
        fn properties(&self, kwargs: KwArgs, #[viper(skip)] vm: &VirtualMachine) -> PyResult<()> {
            for (key, value) in kwargs {
                self.property(key, value, vm)?
            }
            Ok(())
        }

        /// Adds a file denoted by the given string to the report.
        ///
        /// # Example
        ///
        /// ``` python
        /// self.report.file("example.xml")
        /// ```
        #[pymethod]
        fn file(&self, value: PyObjectRef, #[viper(skip)] vm: &VirtualMachine) -> PyResult<()> {
            
            if let Some(value) = value.downcast_ref::<PyStr>() {
                self.collector.set_file_property(value.to_string())
            }
            else {
                Err(unsupported_file_value_type(vm))
            }
        }

        /// Adds the files denoted by the given list of strings to the report.
        ///
        /// # Example
        ///
        /// ```python
        /// self.report.files("foo/bar.txt", "fubar.txt")
        /// ```
        #[pymethod]
        fn files(&self, args: PosArgs, #[viper(skip)] vm: &VirtualMachine) -> PyResult<()> {
            for value in args {
                self.file(value, vm)?
            }
            Ok(())
        }
    }

    impl Debug for PyReportProperties {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "PyReportProperties")
        }
    }

    fn unsupported_property_value_type(name: String, vm: &VirtualMachine) -> PyBaseExceptionRef {
        vm.new_type_error(format!("Unsupported value type for property '{name}'!"))
    }

    fn unsupported_file_value_type(vm: &VirtualMachine) -> PyBaseExceptionRef {
        vm.new_type_error(String::from("Unsupported value type for file property!"))
    }
}
