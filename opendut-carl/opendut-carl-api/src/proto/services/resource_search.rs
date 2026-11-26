use opendut_util::proto::{ConversionError, ConversionErrorBuilder};
use crate::carl::resource_search::SearchError;

tonic::include_proto!("opendut.carl.services.resource_search");

impl From<SearchError> for SearchFailure {
    fn from(error: SearchError) -> Self {
        let proto_error = match error {
            SearchError::Internal { cause } => {
                search_failure::Error::Internal(SearchFailureInternal {
                    cause
                })
            }
        };
        SearchFailure {
            error: Some(proto_error)
        }
    }
}

impl TryFrom<SearchFailure> for SearchError {
    type Error = ConversionError;
    fn try_from(failure: SearchFailure) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<SearchFailure, SearchError>;
        let error = failure.error
            .ok_or_else(|| ErrorBuilder::field_not_set("error"))?;
        let error = match error {
            search_failure::Error::Internal(error) => {
                error.try_into()?
            }
        };
        Ok(error)
    }
}

impl TryFrom<SearchFailureInternal> for SearchError {
    type Error = ConversionError;
    fn try_from(failure: SearchFailureInternal) -> Result<Self, Self::Error> {
        Ok(SearchError::Internal { cause: failure.cause })
    }
}
