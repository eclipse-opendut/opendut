use std::fmt::Debug;
use std::marker::PhantomData;

use crate::persistence::error::PersistenceResult;
use crate::persistence::Storage;
use crate::resources::resource::Resource;

pub mod cluster_configuration;
pub mod cluster_deployment;
pub mod device_descriptor;
pub mod peer_configuration;
pub mod peer_configuration2;
pub mod peer_descriptor;
pub mod peer_state;
mod persistable;

pub trait Persistable: Send + Sync + Sized + Debug + Resource {
    fn insert(self, id: Self::Id, storage: &mut Storage) -> PersistenceResult<()>;

    fn get(id: Self::Id, storage: &Storage) -> PersistenceResult<Option<Self>>;

    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>>;
}

#[derive(Debug, thiserror::Error)]
#[error("Could not convert persistable from `{}` to `{}`:\n  {cause}", std::any::type_name::<From>(), std::any::type_name::<To>())]
pub struct PersistableConversionError<From, To> {
    _from: PhantomData<From>,
    _to: PhantomData<To>,
    #[source] cause: Box<dyn std::error::Error + Send + Sync>,
}
impl<From, To> PersistableConversionError<From, To> {
    pub fn new(cause: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self {
            _from: PhantomData,
            _to: PhantomData,
            cause,
        }
    }
}
