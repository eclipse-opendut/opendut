use std::any::{Any, TypeId};
use std::collections::HashMap;

use opendut_types::resources::Id;

use crate::resources::{IntoId, Iter, IterMut, Resource, Update};
use crate::resources::storage::ResourcesStorageApi;

#[derive(Default)]
pub struct ResourcesMemoryStorage {
    storage: HashMap<TypeId, HashMap<Id, Box<dyn Any + Send + Sync>>>,
}
impl ResourcesStorageApi for ResourcesMemoryStorage {

    fn insert<R>(&mut self, id: impl IntoId<R>, resource: R) -> Option<R>
    where R: Resource {
        let column = self.storage
            .entry(TypeId::of::<R>())
            .or_default();
        column.insert(id.into_id(), Box::new(resource))
            .and_then(|old_value| old_value
                .downcast()
                .map(|value| *value)
                .ok()
            )
    }

    fn update<R>(&mut self, id: impl IntoId<R>) -> Update<R>
    where R: Resource {
        let column = self.storage
            .entry(TypeId::of::<R>())
            .or_default();
        Update {
            id: id.into_id(),
            column,
            marker: Default::default(),
        }
    }

    fn remove<R>(&mut self, id: impl IntoId<R>) -> Option<R>
    where R: Resource {
        let type_id = TypeId::of::<R>();
        let column = self.column_mut_of::<R>()?;
        let result = column.remove(&id.into_id())
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

    fn get<R>(&self, id: impl IntoId<R>) -> Option<R>
    where R: Resource + Clone {
        let column = self.column_of::<R>()?;
        column.get(&id.into_id())
            .and_then(|resource| resource
                .downcast_ref()
                .cloned()
            )
    }

    fn iter<R>(&self) -> Iter<R>
    where R: Resource {
        Iter::new(self.column_of::<R>().map(HashMap::values))
    }

    fn iter_mut<R>(&mut self) -> IterMut<R>
    where R: Resource {
        IterMut::new(self.column_mut_of::<R>().map(HashMap::values_mut))
    }
}
impl ResourcesMemoryStorage {
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
impl ResourcesMemoryStorage {
    pub fn contains<R>(&self, id: impl IntoId<R>) -> bool
    where R: Resource {
        if let Some(column) = self.column_of::<R>() {
            column.contains_key(&id.into_id())
        }
        else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }
}
