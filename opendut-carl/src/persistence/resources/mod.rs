use crate::persistence::error::PersistenceResult;
use crate::persistence::Storage;
use crate::resources::resource::Resource;
use std::fmt::Debug;

pub mod cluster_configuration;
pub mod cluster_deployment;
pub mod peer_configuration;
pub mod peer_configuration2;
pub mod peer_descriptor;
pub mod peer_state;

pub trait Persistable: Send + Sync + Sized + Debug + Resource {
    fn insert(self, id: Self::Id, storage: &mut Storage) -> PersistenceResult<()>;

    fn remove(id: Self::Id, storage: &mut Storage) -> PersistenceResult<Option<Self>>;

    fn get(id: Self::Id, storage: &Storage) -> PersistenceResult<Option<Self>>;

    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>>;
}
