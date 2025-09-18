use crate::compile::{ParameterInfo, Test, TestSuite};
use crate::runtime::types::compile::code::SourceCode;
use crate::runtime::types::compile::inspect::InspectionError;
use crate::runtime::types::compile::metadata::{Metadata, MetadataError};
use crate::runtime::types::compile::parameters::{ParameterDescriptor, ParameterDescriptors, ParameterError};
use crate::runtime::types::compile::suite::TestCase;
use crate::runtime::types::naming::{TestCaseIdentifier, TestIdentifier, TestSuiteIdentifier};
use crate::runtime::types::py::error::PythonReflectionError;
use rustpython_vm::builtins::{PyClassMethod, PyModule, PyStr, PyType};
use rustpython_vm::class::StaticType;
use rustpython_vm::{AsObject, Interpreter};
use rustpython_vm::{PyObjectRef, PyRef};
use viper_py::metadata::metadata::PyMetadata;
use viper_py::parameters::parameters::{PyBooleanParameterDescriptor, PyNumberParameterDescriptor, PyPeerInterfaceParameterDescriptor, PyTextParameterDescriptor};
use viper_py::unittest::unittest::TestCase as PyTestCase;

pub fn inspect(source_code: SourceCode, module: PyRef<PyModule>, interpreter: Interpreter) -> Result<(Metadata, ParameterDescriptors, TestSuite), InspectionError> {
    let SourceCode { identifier, code: _code, version } = source_code;
    let (cases, metadata, parameters) = traverse_code(&identifier, &module, &interpreter)?;
    Ok((metadata, parameters, TestSuite { identifier, version, interpreter, module, cases }))
}

fn traverse_code(test_suite_name: &TestSuiteIdentifier, py_module: &PyRef<PyModule>, interpreter: &Interpreter) -> Result<(Vec<TestCase>, Metadata, ParameterDescriptors), InspectionError> {

    let test_case_base_class = PyTestCase::static_type();

    let mut metadata = Option::<Metadata>::None;
    let mut parameters = ParameterDescriptors::new();
    let mut test_cases = Vec::<TestCase>::new();

    for (key, value) in py_module.dict().into_iter() {
        if let Some(ty) = value.payload::<PyType>() {
            let test_case_name = key.downcast_ref::<PyStr>()
                .ok_or_else(|| PythonReflectionError::new_downcast_error(&key, "PyStr"))
                .expect("downcast to `PyStr` for the `TestCase` name")
                .to_string();
            let test_case_name = TestCaseIdentifier::new(test_suite_name, &test_case_name);
            let is_test_class = ty.bases.read().iter().any(|class| class.is(test_case_base_class));
            if is_test_class {
                test_cases.push(make_test_case(ty, test_case_name, interpreter))
            }
        }
        else if let Some(data) = value.payload::<PyMetadata>() {
            metadata.replace(make_metadata(&key, data) // TODO: If returned (old) value is `Option::Some`, there are multiple metadata variables of which the user should be warned.
                .map_err(InspectionError::new_invalid_metadata_error)?);
        }
        else if let Some(parameter) = value.payload::<PyBooleanParameterDescriptor>() {
            parameters.push(make_boolean_parameter(&key, parameter)
                .map_err(InspectionError::new_invalid_parameter_error)?);
        }
        else if let Some(parameter) = value.payload::<PyNumberParameterDescriptor>() {
            parameters.push(make_number_parameter(&key, parameter)
                .map_err(InspectionError::new_invalid_parameter_error)?);
        }
        else if let Some(parameter) = value.payload::<PyTextParameterDescriptor>() {
            parameters.push(make_text_parameter(&key, parameter)
                .map_err(InspectionError::new_invalid_parameter_error)?);
        }
        else if let Some(_parameter) = value.payload::<PyPeerInterfaceParameterDescriptor>() {
            todo!()
        }
    }

    Ok((test_cases, metadata.unwrap_or_default(), parameters))
}

fn make_test_case(test_type: &PyType, test_case_name: TestCaseIdentifier, interpreter: &Interpreter) -> TestCase {
    const FUNCTION_ATTRIBUTE_NAME: &str = "__func__";
    interpreter.enter(|vm| {
        let mut description: Option<String> = None;
        let mut setup_fn: Option<PyObjectRef> = None;
        let mut teardown_fn: Option<PyObjectRef> = None;
        let mut setup_class_fn: Option<PyObjectRef> = None;
        let mut teardown_class_fn: Option<PyObjectRef> = None;
        let mut tests = Vec::<Test>::new();

        for (key, value) in test_type.get_attributes() {
            let name = key.to_string();

            #[allow(clippy::collapsible_if)]
            if value.is_callable() {
                if name.starts_with("test") {
                    let identifier = TestIdentifier::new(&test_case_name, &name);
                    tests.push(Test { identifier, function: value.clone() })
                } else if name == "setUp" {
                    setup_fn = Some(value.clone());
                } else if name == "tearDown" {
                    teardown_fn = Some(value.clone());
                }
            } else if value.payload_is::<PyClassMethod>() {
                if name == "setUpClass" {
                    setup_class_fn = Some(value.get_attr(FUNCTION_ATTRIBUTE_NAME, vm)
                        .map_err(|_| PythonReflectionError::new_no_such_attribute_error(FUNCTION_ATTRIBUTE_NAME))
                        .expect("setUpClass is no Function in PyClassMethod"));
                } else if name == "tearDownClass" {
                    teardown_class_fn = Some(value.get_attr(FUNCTION_ATTRIBUTE_NAME, vm)
                        .map_err(|_| PythonReflectionError::new_no_such_attribute_error(FUNCTION_ATTRIBUTE_NAME))
                        .expect("tearDownClass is no Function in PyClassMethod"));
                }
            } else if value.payload_is::<PyStr>() {
                if name == "__doc__" {
                    description = Some(value.downcast_ref::<PyStr>()
                        .expect("downcast to `PyStr` for description")
                        .to_string());
                }
            }
        }

        TestCase { identifier: test_case_name, description, setup_fn, teardown_fn, setup_class_fn, teardown_class_fn, tests }
    })
}

fn make_metadata(key: &PyObjectRef, _metadata: &PyMetadata) -> Result<Metadata, MetadataError> {
    let _key = key.downcast_ref::<PyStr>()
        .ok_or_else(|| PythonReflectionError::new_downcast_error(key, "PyStr"))
        .expect("downcast to `PyStr` for metadata key")
        .to_string();
    let mut metadata = Metadata::default();
    for (attr_key, attr_value) in &_metadata.attributes {
        match attr_key.as_str() {
            "display_name" => {
                if let Some(description) = attr_value.downcast_ref::<PyStr>() {
                    metadata.display_name = Some(description.to_string());
                }
                else {
                    return Err(MetadataError::new_wrong_attribute_type_error("display_name", "String"))
                }
            }
            "description" => {
                if let Some(description) = attr_value.downcast_ref::<PyStr>() {
                    metadata.description = Some(description.to_string());
                }
                else {
                    return Err(MetadataError::new_wrong_attribute_type_error("description", "String"))
                }
            }
            "selector" => {
                // TODO: Metadata attribute 'selector' is not supported yet
            }
            &_ => {
                return Err(MetadataError::new_unknown_attribute_error(attr_key));
            }
        }
    }
    Ok(metadata)
}

fn make_boolean_parameter(key: &PyObjectRef, parameter: &PyBooleanParameterDescriptor) -> Result<ParameterDescriptor, ParameterError> {
    let key = key.downcast_ref::<PyStr>().expect("downcast to `PyStr` for parameter key").to_string();
    Ok(ParameterDescriptor::BooleanParameter {
        key,
        name: Clone::clone(&parameter.name).try_into()?,
        info: ParameterInfo { display_name: Clone::clone(&parameter.display_name), description: Clone::clone(&parameter.description) },
        default: parameter.default,
    })
}

fn make_number_parameter(key: &PyObjectRef, parameter: &PyNumberParameterDescriptor) -> Result<ParameterDescriptor, ParameterError> {
    let key = key.downcast_ref::<PyStr>().expect("downcast to `PyStr` for parameter key").to_string();
    Ok(ParameterDescriptor::NumberParameter {
        key,
        name: Clone::clone(&parameter.name).try_into()?,
        info: ParameterInfo { display_name: Clone::clone(&parameter.display_name), description: Clone::clone(&parameter.description) },
        default: parameter.default,
        min: parameter.min,
        max: parameter.max,
    })
}

fn make_text_parameter(key: &PyObjectRef, parameter: &PyTextParameterDescriptor) -> Result<ParameterDescriptor, ParameterError> {
    let key = key.downcast_ref::<PyStr>().expect("downcast to `PyStr` for parameter key").to_string();
    Ok(ParameterDescriptor::TextParameter {
        key,
        name: Clone::clone(&parameter.name).try_into()?,
        info: ParameterInfo { display_name: Clone::clone(&parameter.display_name), description: Clone::clone(&parameter.description) },
        default: Clone::clone(&parameter.default),
        max: parameter.max,
    })
}
