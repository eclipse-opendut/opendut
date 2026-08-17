use crate::runtime::types::compile::metadata::MetadataError;
use crate::runtime::types::compile::parameters::ParameterError;
use crate::runtime::types::compile::filter::FilterError;

#[derive(Debug)]
pub enum InspectionError {
    Metadata(MetadataError),
    Parameter(ParameterError),
    Filter(FilterError)
}

#[allow(dead_code)]
impl InspectionError {

    pub(crate) fn new_invalid_metadata_error(
        source: MetadataError
    ) -> Self {
        Self::Metadata(source)
    }

    pub(crate) fn new_invalid_parameter_error(
        source: ParameterError
    ) -> Self {
        Self::Parameter(source)
    }

    pub(crate) fn new_invalid_filter_error(
        source: FilterError
    ) -> Self { Self::Filter(source) }
}
