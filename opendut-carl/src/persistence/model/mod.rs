use std::fmt::Debug;
use std::marker::PhantomData;
use opendut_types::resources::Id;
use crate::persistence::database::Db;
use crate::resources::resource::Resource;

pub mod cluster_configuration;
pub mod cluster_deployment;
pub mod device_descriptor;
pub mod peer_configuration;
pub mod peer_configuration2;
pub mod peer_descriptor;
pub mod peer_state;

pub trait Persistable<R>
where
    R: Resource,
    Self: Debug,
    Self: From<R>,
    Self: TryInto<R, Error=PersistableConversionError<Self, R>>,
{
    fn insert(&self, db: Db) -> Option<Self>;

    fn get(id: &Id, db: Db) -> Option<Self>;
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
