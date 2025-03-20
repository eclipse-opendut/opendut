use std::collections::HashMap;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::Storage;
use std::fmt::Debug;
use crate::resource::api::Resource;

pub mod cluster_configuration;
pub mod cluster_deployment;
pub mod old_peer_configuration;
pub mod peer_configuration;
pub mod peer_descriptor;
mod peer_connection_state;

pub trait Persistable: Send + Sync + Sized + Debug + Resource {
    fn insert(self, id: Self::Id, storage: &mut Storage) -> PersistenceResult<()>;

    fn remove(id: Self::Id, storage: &mut Storage) -> PersistenceResult<Option<Self>>;

    fn get(id: Self::Id, storage: &Storage) -> PersistenceResult<Option<Self>>;

    fn list(storage: &Storage) -> PersistenceResult<HashMap<Self::Id, Self>>;
}
