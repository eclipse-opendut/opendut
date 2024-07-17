use std::fmt::Debug;
use std::marker::PhantomData;
use crate::persistence::database::Db;

pub mod cluster_configuration;
pub mod cluster_deployment;
pub mod device_descriptor;
pub mod peer_configuration;
pub mod peer_configuration2;
pub mod peer_descriptor;
pub mod peer_state;

pub trait Persistable<Model>: From<Model> + Debug
where
    Model: TryFrom<Self, Error=PersistableConversionError<Self, Model>>
{
    fn insert(&self, db: Db) -> Option<Self>;
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
