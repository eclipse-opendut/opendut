use uuid::Uuid;
use crate::resource::versioning::Revision;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum CommitError {
    #[error("Resource <{uuid}> cannot be committed, because the given parent revision '{}' is not the current head revision '{}'!", actual.parent, head.current)]
    InvalidParentRevision {
        uuid: Uuid,
        actual: Revision,
        head: Revision,
    },
    #[error("Resource <{uuid}> cannot be committed, because the given parent revision '{}' does not exist!", actual.parent)]
    UnknownParentRevision {
        uuid: Uuid,
        actual: Revision,
    },
    #[error("Resource <{uuid}> cannot be committed, due to a serialization error:\n  {cause}")]
    SerializationError {
        uuid: Uuid,
        cause: String,
    }
}

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum HeadError {
    #[error("The resource <{uuid}> does not exist!")]
    ResourceNotFound {
        uuid: Uuid,
    }
}

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum GetError {
    #[error("The resource <{uuid}> does not exist!")]
    ResourceNotFound {
        uuid: Uuid,
    },
    #[error("The revision '{}' for resource <{uuid}> does not exist!", revision.current)]
    UnknownRevision {
        uuid: Uuid,
        revision: Revision,
    },
}
