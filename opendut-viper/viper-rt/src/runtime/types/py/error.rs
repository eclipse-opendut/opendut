#[cfg(feature = "py")]
use rustpython_vm as vm;

#[derive(Debug)]
#[allow(dead_code)]
#[non_exhaustive]
pub enum PythonReflectionError {
    NoSuchAttribute {
        attribute_name: String,
    },
    AttributeNotWritable {
        owner_name: String,
        attribute_name: String,
    },
    Downcast {
        source_type: String,
        target_type: String,
    }
}

impl PythonReflectionError {

    #[cfg(feature = "py")]
    pub(crate) fn new_no_such_attribute_error(attribute_name: impl Into<String>) -> Self {
        Self::NoSuchAttribute {
            attribute_name: attribute_name.into(),
        }
    }

    #[cfg(feature = "py")]
    pub(crate) fn new_attribute_not_writable_error(owner_name: impl Into<String>, attribute_name: impl Into<String>) -> Self {
        Self::AttributeNotWritable {
            owner_name: owner_name.into(),
            attribute_name: attribute_name.into(),
        }
    }

    #[cfg(feature = "py")]
    pub(crate) fn new_downcast_error(obj: &vm::PyObjectRef, target_type: impl Into<String>) -> Self {
        Self::Downcast {
            source_type: obj.class().name().to_string(),
            target_type: target_type.into(),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
#[non_exhaustive]
pub struct PythonRuntimeError {
    pub details: String,
}

impl PythonRuntimeError {

    #[cfg(feature = "py")]
    pub(crate) fn from_base_exception(exception: vm::PyRef<vm::builtins::PyBaseException>) -> Self {
        let args = exception.args().iter().map(|arg| format!("{arg:?}")).collect::<Vec<String>>().join(", ");
        let details = format!("error: {exception:?}: {args:?}");
        Self {
            details,
        }
    }
}
