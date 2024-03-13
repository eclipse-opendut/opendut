use std::mem::size_of;
use uuid::Uuid;

use crate::resource::versioning::{Revision, RevisionHash, Versioned};

#[derive(Clone, Debug)]
pub struct Change {
    kind: ChangeKind,
    uuid: Uuid,
    revision: Revision,
    bytes: Vec<u8>,
}

impl Change {
    pub fn new(kind: ChangeKind, uuid: Uuid, revision: Revision, bytes: Vec<u8>) -> Self {
        Self { kind, uuid, revision, bytes }
    }

    pub fn nothing(uuid: Uuid, revision: Revision) -> Self {
        Self {
            kind: ChangeKind::Nothing,
            uuid,
            revision,
            bytes: Vec::new(),
        }
    }

    pub fn created(uuid: Uuid, revision: Revision, bytes: Vec<u8>) -> Self {
        Self {
            kind: ChangeKind::Created,
            uuid,
            revision,
            bytes,
        }
    }

    pub fn updated(uuid: Uuid, revision: Revision, bytes: Vec<u8>) -> Self {
        Self {
            kind: ChangeKind::Updated,
            uuid,
            revision,
            bytes,
        }
    }

    pub fn removed(uuid: Uuid, revision: Revision) -> Self {
        Self {
            kind: ChangeKind::Removed,
            uuid,
            revision,
            bytes: Vec::new(),
        }
    }

    pub fn uuid(&self) -> &uuid::Uuid {
        &self.uuid
    }

    pub fn revision(&self) -> &Revision {
        &self.revision
    }

    pub fn revision_hash(&self) -> &RevisionHash {
        &self.revision.current_hash()
    }

    pub fn revision_parent(&self) -> &RevisionHash {
        &self.revision.parent_hash()
    }

    pub fn kind(&self) -> &ChangeKind {
        &self.kind
    }

    pub fn into_patch(self) -> Patch {

        let mut bytes = Vec::<u8>::with_capacity(size_of::<u8>() + size_of::<uuid::Uuid>() + self.bytes.len());

        bytes.push(self.kind.tag());
        bytes.extend_from_slice(self.uuid().as_bytes());
        bytes.extend(self.bytes);

        Patch::new(bytes)
    }

    // pub fn into_resource(self) -> R {
    //     self.resource
    // }

    // pub fn into_resource_ref(self) -> RR {
    //     self.resource.into_resource_ref()
    // }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ChangeKind {
    Nothing,
    Created,
    Updated,
    Removed,
}

impl ChangeKind {

    pub(crate) fn try_from_tag(tag: u8) -> Result<Self, ()> {
        match tag {
            1 => Ok(ChangeKind::Nothing),
            2 => Ok(ChangeKind::Created),
            3 => Ok(ChangeKind::Updated),
            4 => Ok(ChangeKind::Removed),
            _ => Err(()),
        }
    }

    pub(crate) fn tag(&self) -> u8 {
        match self {
            ChangeKind::Nothing => 1,
            ChangeKind::Created => 2,
            ChangeKind::Updated => 3,
            ChangeKind::Removed => 4,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Patch {
    pub bytes: Vec<u8>
}

impl Patch {
    pub fn new(bytes: Vec<u8>) -> Self {
        Patch { bytes }
    }
}
