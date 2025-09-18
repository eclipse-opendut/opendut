use crate::runtime::types::compile::metadata::MetadataError;
use crate::runtime::types::compile::parameters::ParameterError;

#[derive(Debug)]
pub enum InspectionError {
    MetadataError(MetadataError),
    ParameterError(ParameterError)
}

impl InspectionError {

    pub(crate) fn new_invalid_metadata_error(
        cause: MetadataError
    ) -> Self {
        Self::MetadataError(cause)
    }

    pub(crate) fn new_invalid_parameter_error(
        cause: ParameterError
    ) -> Self {
        Self::ParameterError(cause)
    }
}
