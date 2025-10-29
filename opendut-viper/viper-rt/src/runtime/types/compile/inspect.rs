use crate::runtime::types::compile::metadata::MetadataError;
use crate::runtime::types::compile::parameters::ParameterError;
use crate::runtime::types::compile::filter::FilterError;

#[derive(Debug)]
pub enum InspectionError {
    Metadata(MetadataError),
    Parameter(ParameterError),
    Filter(FilterError)
}

impl InspectionError {

    pub(crate) fn new_invalid_metadata_error(
        cause: MetadataError
    ) -> Self {
        Self::Metadata(cause)
    }

    pub(crate) fn new_invalid_parameter_error(
        cause: ParameterError
    ) -> Self {
        Self::Parameter(cause)
    }

    pub(crate) fn new_invalid_filter_error(
        cause: FilterError
    ) -> Self { Self::Filter(cause) }
}
