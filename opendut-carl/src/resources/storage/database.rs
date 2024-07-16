use crate::persistence::model::Persistable;
use crate::resources::{IntoId, Iter, IterMut, Update};
use crate::resources::storage::{Resource, ResourcesStorageApi};

pub struct ResourcesDatabaseStorage;
impl ResourcesStorageApi for ResourcesDatabaseStorage {
    fn insert<R>(&mut self, id: impl IntoId<R>, resource: R) -> Option<R>
    where R: Resource {
        let persistable = R::Persistable::from(resource);

        let result = persistable.insert(); //TODO pass client, id, etc.?

        result.map(R::try_from)
            .transpose()
            .unwrap_or_else(|_| panic!("Failed to insert resource into database: {persistable:?}"))
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
