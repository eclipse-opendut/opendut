use std::any::{Any, TypeId};
use std::collections::HashMap;

use opendut_types::resources::Id;
use crate::persistence::error::PersistenceResult;
use crate::resources::{Resource, Update};
use crate::resources::storage::ResourcesStorageApi;

#[derive(Default)]
pub struct VolatileResourcesStorage {
    storage: HashMap<TypeId, HashMap<Id, Box<dyn Any + Send + Sync>>>,
}
impl ResourcesStorageApi for VolatileResourcesStorage {

    fn insert<R>(&mut self, id: Id, resource: R) -> PersistenceResult<()>
    where R: Resource {
        let column = self.storage
            .entry(TypeId::of::<R>())
            .or_default();
        column.insert(id, Box::new(resource));
        Ok(())
    }

    fn update<R>(&mut self, id: Id) -> Update<R>
    where R: Resource {
        let column = self.storage
            .entry(TypeId::of::<R>())
            .or_default();
        Update {
            id,
            column,
            marker: Default::default(),
        }
    }

    fn remove<R>(&mut self, id: Id) -> Option<R>
    where R: Resource {
        let type_id = TypeId::of::<R>();
        let column = self.column_mut_of::<R>()?;
        let result = column.remove(&id)
            .and_then(|old_value| old_value
                .downcast()
                .map(|value| *value)
                .ok()
            );
        if column.is_empty() {
            self.storage.remove(&type_id);
        }
        result
    }

    fn get<R>(&self, id: Id) -> PersistenceResult<Option<R>>
    where R: Resource + Clone {
        let result = self.column_of::<R>()
            .and_then(|column| column.get(&id))
            .and_then(|resource| resource.downcast_ref().cloned());
        Ok(result)
    }

    fn list<R>(&self) -> PersistenceResult<Vec<R>>
    where R: Resource {
        let result = match self.column_of::<R>() {
            Some(column) => {
                column.values()
                    .map(|value| value
                        .downcast_ref::<R>()
                        .cloned()
                        .expect("It should always be possible to cast the stored data back to its own type while building an iterator.")
                    )
                    .collect()
            }
            None => Vec::new()
        };
        Ok(result)
    }
}
impl VolatileResourcesStorage {
    fn column_of<R>(&self) -> Option<&HashMap<Id, Box<dyn Any + Send + Sync>>>
    where R: Resource {
        self.storage.get(&TypeId::of::<R>())
    }

    fn column_mut_of<R>(&mut self) -> Option<&mut HashMap<Id, Box<dyn Any + Send + Sync>>>
    where R: Resource {
        self.storage.get_mut(&TypeId::of::<R>())
    }
}

#[cfg(test)]
impl VolatileResourcesStorage {
    pub fn contains<R>(&self, id: Id) -> bool
    where R: Resource {
        if let Some(column) = self.column_of::<R>() {
            column.contains_key(&id)
        }
        else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }
}
