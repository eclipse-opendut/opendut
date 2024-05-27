use crate::resources::{IntoId, Iter, IterMut, Update};
use crate::resources::storage::{Resource, ResourcesStorageApi};

pub struct ResourcesDatabaseStorage;
impl ResourcesStorageApi for ResourcesDatabaseStorage {
    fn insert<R>(&mut self, id: impl IntoId<R>, resource: R) -> Option<R>
    where R: Resource {
        todo!()
    }

    fn update<R>(&mut self, id: impl IntoId<R>) -> Update<R>
    where R: Resource {
        todo!()
    }

    fn remove<R>(&mut self, id: impl IntoId<R>) -> Option<R>
    where R: Resource {
        todo!()
    }

    fn get<R>(&self, id: impl IntoId<R>) -> Option<R>
    where R: Resource + Clone {
        todo!()
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
