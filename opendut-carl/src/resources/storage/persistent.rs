use url::Url;

use opendut_types::resources::Id;

use crate::persistence::database::ConnectError;
use crate::persistence::Storage;
use crate::resources::{Iter, IterMut, Update};
use crate::resources::storage::{Resource, ResourcesStorageApi};
use crate::resources::storage::volatile::VolatileResourcesStorage;

pub struct PersistentResourcesStorage {
    storage: Storage,
}
impl PersistentResourcesStorage {
    pub fn connect(url: &Url) -> Result<Self, ConnectError> {
        let db = crate::persistence::database::connect(url)?;
        let memory = VolatileResourcesStorage::default();
        let storage = Storage { db, memory };
        Ok(Self { storage })
    }
}
impl ResourcesStorageApi for PersistentResourcesStorage {
    fn insert<R>(&mut self, id: Id, resource: R)
    where R: Resource {
        resource.insert(id, &mut self.storage);
    }

    fn update<R>(&mut self, id: Id) -> Update<R>
    where R: Resource {
        todo!()
    }

    fn remove<R>(&mut self, id: Id) -> Option<R>
    where R: Resource {
        todo!()
    }

    fn get<R>(&self, id: Id) -> Option<R>
    where R: Resource + Clone {
        R::get(id, &self.storage)
    }

    fn iter<R>(&self) -> Iter<R>
    where R: Resource {
        todo!()
    }

    fn iter_mut<R>(&mut self) -> IterMut<R>
    where R: Resource {
        todo!()
    }
}
