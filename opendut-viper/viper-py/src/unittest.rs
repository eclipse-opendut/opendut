#![allow(clippy::module_inception)]
use rustpython_vm::pymodule;

#[pymodule]
pub mod unittest {
    use rustpython_vm::function::OptionalArg;
    use rustpython_vm::{pyclass, AsObject, PyObjectRef, PyPayload, PyResult, VirtualMachine};
    use std::ops::Not;

    #[pyattr]
    #[pyclass(name)]
    #[derive(Debug, PyPayload)]
    pub struct TestCase {}

    #[pyclass(flags(BASETYPE))] // Enables inheritance
    #[viper_pygen::pygen]
    impl TestCase {

        /// Test that `a` and `b` are equal. If the values do not compare equal, the test will fail.
        #[pymethod(name = "assertEquals")]
        #[viper(name = "assertEquals")]
        fn assert_equals(
            _this: PyObjectRef,
            a: PyObjectRef,
            b: PyObjectRef,
            #[viper(default = "")] message: OptionalArg<String>,
            #[viper(skip)] vm: &VirtualMachine
        ) -> PyResult<()> {
            let is_equals = vm.call_method(&a, "__eq__", vec![b.clone()])?;

            if is_equals.is(&vm.ctx.not_implemented) {
                let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: The objects are not comparable"));
                return Err(vm.new_runtime_error(error_message));
            }

            if is_equals.is_true(vm)? {
                Ok(())
            } else {
                let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: The Objects are not equal"));
                Err(vm.new_runtime_error(error_message))
            }
        }

        /// Test that `a` and `b` are not equal. If the values do compare equal, the test will fail.
        #[pymethod(name = "assertNotEquals")]
        #[viper(name = "assertNotEquals")]
        fn assert_not_equals(
            _this: PyObjectRef,
            a: PyObjectRef,
            b: PyObjectRef,
            #[viper(default = "")] message: OptionalArg<String>,
            #[viper(skip)] vm: &VirtualMachine
        ) -> PyResult<()> {
            let is_equals = vm.call_method(&a, "__eq__", vec![b])?;


            if is_equals.is(&vm.ctx.not_implemented) {
                let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: The objects are not comparable"));
                return Err(vm.new_runtime_error(error_message));
            }

            if is_equals.is_true(vm)?.not() {
                Ok(())
            } else {
                let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: Equal objects"));
                Err(vm.new_runtime_error(error_message))
            }
        }

        /// Test that given `expression` is `True`.
        #[pymethod(name = "assertTrue")]
        #[viper(name = "assertTrue")]
        fn assert_true(
            _this: PyObjectRef,
            expression: bool,
            #[viper(default = "")] message: OptionalArg<String>,
            #[viper(skip)] vm: &VirtualMachine
        ) -> PyResult<()> {
            let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: The expression is false"));

            if expression {
                Ok(())
            } else {
                Err(vm.new_runtime_error(error_message))
            }
        }

        /// Test that given `expression` is `False`.
        #[pymethod(name = "assertFalse")]
        #[viper(name = "assertFalse")]
        fn assert_false(
            _this: PyObjectRef,
            expression: bool,
            #[viper(default = "")] message: OptionalArg<String>,
            #[viper(skip)] vm: &VirtualMachine
        ) -> PyResult<()> {
            let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: The expression is true"));

            if expression.not() {
                Ok(())
            } else {
                Err(vm.new_runtime_error(error_message))
            }
        }

        /// Test that `a` and `b` are the same object.
        #[pymethod(name = "assertIs")]
        #[viper(name = "assertIs")]
        fn assert_is(
            _this: PyObjectRef,
            a: PyObjectRef,
            b: PyObjectRef,
            #[viper(default = "")] message: OptionalArg<String>,
            #[viper(skip)] vm: &VirtualMachine
        ) -> PyResult<()> {
            let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: Different objects"));

            if a.is(&b) {
                Ok(())
            } else {
                Err(vm.new_runtime_error(error_message))
            }
        }

        /// Test that `a` and `b` are different objects.
        #[pymethod(name = "assertIsNot")]
        #[viper(name = "assertIsNot")]
        fn assert_is_not(
            _this: PyObjectRef,
            a: PyObjectRef,
            b: PyObjectRef,
            #[viper(default = "")] message: OptionalArg<String>,
            #[viper(skip)] vm: &VirtualMachine
        ) -> PyResult<()> {
            let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: Different objects"));

            if a.is(&b).not() {
                Ok(())
            } else {
                Err(vm.new_runtime_error(error_message))
            }
        }

        /// Test that `object` is None.
        #[pymethod(name = "assertIsNone")]
        #[viper(name = "assertIsNone")]
        fn assert_is_none(
            _this: PyObjectRef,
            object: PyObjectRef,
            #[viper(default = "")] message: OptionalArg<String>,
            #[viper(skip)] vm: &VirtualMachine
        ) -> PyResult<()> {
            let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: Object is not none"));

            if vm.is_none(&object) {
                Ok(())
            } else {
                Err(vm.new_runtime_error(error_message))
            }
        }

        /// Test that `object` is not None.
        #[pymethod(name = "assertIsNotNone")]
        #[viper(name = "assertIsNotNone")]
        fn assert_is_not_none(
            _this: PyObjectRef,
            object: PyObjectRef,
            #[viper(default = "")] message: OptionalArg<String>,
            #[viper(skip)] vm: &VirtualMachine
        ) -> PyResult<()> {
            let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: The objects is none"));

            if vm.is_none(&object).not() {
                Ok(())
            } else {
                Err(vm.new_runtime_error(error_message))
            }
        }

        /// Test that `element` is in `container`.
        #[pymethod(name = "assertIn")]
        #[viper(name = "assertIn")]
        fn assert_in(
            _this: PyObjectRef,
            element: PyObjectRef,
            container: PyObjectRef,
            #[viper(default = "")] message: OptionalArg<String>,
            #[viper(skip)] vm: &VirtualMachine
        ) -> PyResult<()> {
            let result = vm.call_method(&container, "__contains__", vec![element])?;
            let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: Element is not in the container"));

            if result.is_true(vm)? {
                Ok(())
            } else {
                Err(vm.new_runtime_error(error_message))
            }
        }

        /// Test that `element` is not in `container`.
        #[pymethod(name = "assertNotIn")]
        #[viper(name = "assertNotIn")]
        fn assert_not_in(
            _this: PyObjectRef,
            element: PyObjectRef,
            container: PyObjectRef,
            #[viper(default = "")] message: OptionalArg<String>,
            #[viper(skip)] vm: &VirtualMachine
        ) -> PyResult<()> {
            let result = vm.call_method(&container, "__contains__", vec![element])?;
            let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: Element is in the container"));

            if result.is_true(vm)?.not() {
                Ok(())
            } else {
                Err(vm.new_runtime_error(error_message))
            }
        }

        /// Test that `object` is an instance of `cls`.
        #[pymethod(name = "assertIsInstance")]
        #[viper(name = "assertIsInstance")]
        fn assert_is_instance(
            _this: PyObjectRef,
            object: PyObjectRef,
            cls: PyObjectRef,
            #[viper(default = "")] message: OptionalArg<String>,
            #[viper(skip)] vm: &VirtualMachine
        ) -> PyResult<()> {
            let is_instance = object.is_instance(&cls, vm)?;
            let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: Object is not an instance"));

            if is_instance {
                Ok(())
            } else {
                Err(vm.new_runtime_error(error_message))
            }
        }

        /// Test that `object` is not an instance of `cls`.
        #[pymethod(name = "assertIsNotInstance")]
        #[viper(name = "assertIsNotInstance")]
        fn assert_is_not_instance(
            _this: PyObjectRef,
            object: PyObjectRef,
            cls: PyObjectRef,
            #[viper(default = "")] message: OptionalArg<String>,
            #[viper(skip)] vm: &VirtualMachine
        ) -> PyResult<()> {
            let is_instance = object.is_instance(&cls, vm)?;
            let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: Object is an instance"));

            if is_instance.not() {
                Ok(())
            } else {
                Err(vm.new_runtime_error(error_message))
            }
        }

        /// Test that `left` is greater than `right`, otherwise the test will fail.
        #[pymethod(name = "assertGreater")]
        #[viper(name = "assertGreater")]
        fn assert_greater(
            _this: PyObjectRef,
            left: PyObjectRef,
            right: PyObjectRef,
            #[viper(default = "")] message: OptionalArg<String>,
            #[viper(skip)] vm: &VirtualMachine
        ) -> PyResult<()> {
            let result = vm.call_method(&left, "__gt__", vec![right])?;

            if result.is(&vm.ctx.not_implemented) {
                let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: The objects are not comparable"));
                return Err(vm.new_runtime_error(error_message));
            }

            if result.is_true(vm)? {
                Ok(())
            } else {
                let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: The left object is less than the right one"));
                Err(vm.new_runtime_error(error_message))
            }
        }

        /// Test that `left` is less than `right`, otherwise the test will fail.
        #[pymethod(name = "assertLess")]
        #[viper(name = "assertLess")]
        fn assert_less(
            _this: PyObjectRef,
            left: PyObjectRef,
            right: PyObjectRef,
            #[viper(default = "")] message: OptionalArg<String>,
            #[viper(skip)] vm: &VirtualMachine
        ) -> PyResult<()> {
            let result = vm.call_method(&left, "__lt__", vec![right])?;

            if result.is(&vm.ctx.not_implemented) {
                let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: The objects are not comparable"));
                return Err(vm.new_runtime_error(error_message));
            }

            if result.is_true(vm)? {
                Ok(())
            } else {
                let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: The left object is greater than the right one"));
                Err(vm.new_runtime_error(error_message))
            }
        }

        /// Test that `left` is greater than or equal to `right`, otherwise the test will fail.
        #[pymethod(name = "assertGreaterOrEqual")]
        #[viper(name = "assertGreaterOrEqual")]
        fn assert_greater_or_equal(
            _this: PyObjectRef,
            left: PyObjectRef,
            right: PyObjectRef,
            #[viper(default = "")] message: OptionalArg<String>,
            #[viper(skip)] vm: &VirtualMachine
        ) -> PyResult<()> {
            let result = vm.call_method(&left, "__ge__", vec![right])?;

            if result.is(&vm.ctx.not_implemented) {
                let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: The objects are not comparable"));
                return Err(vm.new_runtime_error(error_message));
            }

            if result.is_true(vm)? {
                Ok(())
            } else {
                let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: The left object is less than the right one"));
                Err(vm.new_runtime_error(error_message))
            }
        }

        /// Test that `left` is less than or equal to `right`, otherwise the test will fail.
        #[pymethod(name = "assertLessOrEqual")]
        #[viper(name = "assertLessOrEqual")]
        fn assert_less_or_equal(
            _this: PyObjectRef,
            left: PyObjectRef,
            right: PyObjectRef,
            #[viper(default = "")] message: OptionalArg<String>,
            #[viper(skip)] vm: &VirtualMachine
        ) -> PyResult<()> {
            let result = vm.call_method(&left, "__le__", vec![right])?;

            if result.is(&vm.ctx.not_implemented) {
                let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: The objects are not comparable"));
                return Err(vm.new_runtime_error(error_message));
            }

            if result.is_true(vm)? {
                Ok(())
            } else {
                let error_message = message.unwrap_or_else(||String::from("ASSERTION FAILED: The left object is greater than the right one"));
                Err(vm.new_runtime_error(error_message))
            }
        }

        /// Signals a test failure unconditionally.
        #[pymethod]
        fn fail(
            _this: PyObjectRef,
            #[viper(default = "")] message: OptionalArg<String>,
            #[viper(skip)] vm: &VirtualMachine
        ) -> PyResult<()> {
            let message = message.unwrap_or_else(|| String::from("Test Failed!"));
            Err(vm.new_runtime_error(message))
        }
    }
}
