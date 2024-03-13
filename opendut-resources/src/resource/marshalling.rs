use std::error::Error;

use crate::resource::Resource;

pub type ResourceTag = i32;
pub type MarshallerIdentifier = i32;

pub trait Marshaller<R>
where
    R: Resource
{
    const IDENTIFIER: MarshallerIdentifier;

    fn marshal(&self, resource: &R) -> Result<Vec<u8>, MarshalError>;

    fn unmarshal(&self, bytes: &[u8]) -> Result<R, UnmarshalError>;
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
    #[error("Unknown ResourceTag '{tag}'!")]
    UnknownResourceTag {
        tag: ResourceTag
    },
    #[error("Unmarshalling failure due to error:\n  {cause}")]
    UnmarshallingFailure {
        cause: Box<dyn Error>
    }
}
