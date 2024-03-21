use std::error::Error;
use std::io::Write;

use crate::resource::generic::GenericResource;

pub type ResourceTag = i32;
pub type MarshallerIdentifier = i32;

pub trait Marshaller {

    const IDENTIFIER: MarshallerIdentifier;

    // fn marshal(&self, resource: &dyn GenericResource) -> Result<Vec<u8>, MarshalError>;
    fn marshal<'r, W>(&self, resource: &'r dyn GenericResource, writer: W) -> Result<Vec<u8>, MarshalError>
        where
            W: Write;

    fn unmarshal(&self, bytes: &[u8]) -> Result<Box<dyn GenericResource>, UnmarshalError>;
}

#[derive(Debug, thiserror::Error)]
pub enum MarshalError {
    #[error("Unknown Resource!")]
    UnknownResource,
    #[error("Unknown ResourceTag '{tag}'!")]
    UnknownResourceTag {
        tag: ResourceTag
    },
    #[error("Marshalling failure due to error:\n  {cause}")]
    MarshallingFailure {
        cause: Box<dyn Error>
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UnmarshalError {

    #[error("Unknown resource!")]
    UnknownResource,

    #[error("Unmarshalling failure due to error:\n  {cause}")]
    UnmarshallingFailure {
        cause: Box<dyn Error>
    }
}
