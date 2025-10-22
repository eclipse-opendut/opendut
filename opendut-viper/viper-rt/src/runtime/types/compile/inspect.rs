use crate::runtime::types::compile::metadata::MetadataError;
use crate::runtime::types::compile::parameters::ParameterError;
use crate::runtime::types::compile::filter::FilterError;

#[derive(Debug)]
pub enum InspectionError {
    MetadataError(MetadataError),
    ParameterError(ParameterError),
    FilterError(FilterError)
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

    pub(crate) fn new_invalid_filter_error(
        cause: FilterError
    ) -> Self { Self::FilterError(cause) }
}
