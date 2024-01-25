use uuid::Uuid;

use crate::resource::{Resource, ResourceRef};
use crate::resource::versioning::{BorrowedRevision, BorrowRevision, RevisionHash, Versioned};

#[derive(Clone, Debug)]
pub struct Change<R, RR>
where
    R: Resource<ResourceRef=RR>,
    RR: ResourceRef<R>
{
    resource: R,
    kind: ChangeKind,
}

impl <R, RR> Change<R, RR>
where
    R: Resource<ResourceRef=RR>,
    RR: ResourceRef<R> + Versioned + BorrowRevision
{
    pub fn nothing(resource: R) -> Self {
        Self {
            resource,
            kind: ChangeKind::Nothing,
        }
    }

    pub fn created(resource: R) -> Self {
        Self {
            resource,
            kind: ChangeKind::Created,
        }
    }

    pub fn updated(resource: R) -> Self {
        Self {
            resource,
            kind: ChangeKind::Updated,
        }
    }

    pub fn removed(resource: R) -> Self {
        Self {
            resource,
            kind: ChangeKind::Removed,
        }
    }

    pub fn uuid(&self) -> &Uuid {
        self.resource.resource_ref().uuid()
    }

    pub fn revision(&self) -> BorrowedRevision<RR> {
        self.resource.resource_ref().borrow_revision()
    }

    pub fn revision_hash(&self) -> &RevisionHash {
        self.resource.resource_ref().current_hash()
    }

    pub fn revision_parent(&self) -> &RevisionHash {
        self.resource.resource_ref().parent_hash()
    }

    pub fn resource(&self) -> &R {
        &self.resource
    }

    pub fn resource_ref(&self) -> &RR
    {
        self.resource.resource_ref()
    }

    pub fn kind(&self) -> &ChangeKind {
        &self.kind
    }

    pub fn into_resource(self) -> R {
        self.resource
    }

    pub fn into_resource_ref(self) -> RR {
        self.resource.into_resource_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ChangeKind {
    Nothing,
    Created,
    Updated,
    Removed,
}
