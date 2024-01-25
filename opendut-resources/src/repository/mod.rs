use std::any::Any;
use std::collections::HashMap;
use std::ops::Not;

use circular_buffer::CircularBuffer;
use uuid::Uuid;
use xxhash_rust::xxh3::xxh3_128;

pub use change::{Change, ChangeKind};
pub use errors::{CommitError, GetError, HeadError};
use crate::resource::{Resource, ResourceRef};

use crate::resource::versioning::ROOT_REVISION_HASH;
use crate::resource::versioning::{BorrowRevision, RevisionHash, Revision, ToRevision, Versioned, VersionedMut};

mod errors;
mod change;

#[derive(Default)]
pub struct Repository<const N: usize> {
    index: HashMap<Uuid, CircularBuffer<N, Revision>>,
    store: HashMap<RevisionHash, Box<dyn Any + Send + Sync>>,
}

impl <const N: usize> Repository<N> {

    pub fn new() -> Self {
        Default::default()
    }

    #[cfg(feature = "broker")]
    pub fn commit<R, RR>(&mut self, resource: R) -> Result<Change<R, RR>, CommitError>
    where
        R: Any + Resource<ResourceRef=RR> + Send + Sync,
        RR: ResourceRef<R> + BorrowRevision + ToRevision
    {
        let mut resource = resource;
        let resource_ref = resource.resource_ref();
        let uuid = resource_ref.uuid().to_owned();

        let revisions = self.index
            .entry(resource_ref.uuid().to_owned())
            .or_default();

        let revision = resource.resource_ref().borrow_revision();

        let (parent, is_new) = if let Some(head_revision) = revisions.front() {
            if revision.current_hash() == head_revision.current_hash() {
                let change = Change::nothing(resource);
                return Ok(change);
            }
            if revision.parent_hash() != head_revision.current_hash() {
                return Err(CommitError::InvalidParentRevision {
                    uuid,
                    actual: resource_ref.revision(),
                    head: Clone::clone(head_revision),
                });
            }
            (head_revision.current_hash().to_owned(), false)
        }
        else {
            let parent = *revision.parent_hash();
            if ROOT_REVISION_HASH != parent {
                return Err(CommitError::UnknownParentRevision {
                    uuid,
                    actual: resource_ref.revision(),
                })
            }
            (parent, true)
        };

        resource.resource_ref_mut()
            .clear_revision();

        let hash: RevisionHash = postcard::to_stdvec(&resource)
            .map_err(|cause| CommitError::SerializationError { uuid, cause: cause.to_string() })
            .map(|bytes| RevisionHash::from(xxh3_128(&bytes)))?;

        if hash == parent {
            Ok(Change::nothing(resource))
        }
        else {
            resource.resource_ref_mut()
                .reset_revision(hash, parent);

            let revision = resource.resource_ref().revision();

            if let Err(revision) = revisions.try_push_front(revision) {
                if let Some(oldest_revision) = revisions.pop_back() {
                    self.store.remove(oldest_revision.current_hash());
                }
                revisions.try_push_front(revision)
                    .expect("After removing the oldest revision, the buffer should now have space to push a new revision.")
            }

            self.store.insert(hash, Box::new(Clone::clone(&resource)));

            if is_new {
                Ok(Change::created(resource))
            } else {
                Ok(Change::updated(resource))
            }
        }
    }

    pub fn head<R, RR>(&self, resource_ref: &RR) -> Result<R, HeadError>
    where
        R: Any + Resource,
        RR: ResourceRef<R>
    {
        let uuid = resource_ref.uuid();

        let revision = self.index.get(uuid)
            .and_then(|revisions| revisions.front())
            .ok_or_else(|| HeadError::ResourceNotFound {
                uuid: uuid.to_owned()
            })?;

        let resource = self.store.get(revision.current_hash())
            .expect("If there is a head revision then there should be a resource in the store.")
            .downcast_ref::<R>()
            .expect("Downcast to concrete resource type should be possible")
            .to_owned();

        Ok(resource)
    }

    pub fn get<R, RR>(&self, resource_ref: &RR) -> Result<R, GetError>
    where
        R: Any + Resource,
        RR: ResourceRef<R> + Versioned + BorrowRevision
    {
        let uuid = resource_ref.uuid();
        let revision = resource_ref.borrow_revision();

        if self.index.contains_key(resource_ref.uuid()).not() {
            return Err(GetError::ResourceNotFound {
                uuid: uuid.to_owned(),
            });
        }

        let resource = self.store.get(revision.current_hash())
            .ok_or_else(|| GetError::UnknownRevision {
                uuid: uuid.to_owned(),
                revision: revision.into(),
            })?
            .downcast_ref::<R>()
            .expect("Downcast to concrete resource type should be possible")
            .to_owned();

        Ok(resource)
    }
}
