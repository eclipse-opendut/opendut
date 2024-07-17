use diesel::ConnectionError;
use url::Url;
use crate::persistence::database::Db;
use crate::persistence::model::Persistable;
use crate::resources::{IntoId, Iter, IterMut, Update};
use crate::resources::storage::{Resource, ResourcesStorageApi};

pub struct ResourcesDatabaseStorage {
    db: Db,
}
impl ResourcesDatabaseStorage {
    pub fn connect(url: &Url) -> Result<Self, ConnectionError> {
        let db = crate::persistence::database::connect(url)?;
        Ok(Self { db })
    }
}
impl ResourcesStorageApi for ResourcesDatabaseStorage {
    fn insert<R>(&mut self, id: impl IntoId<R>, resource: R) -> Option<R>
    where R: Resource {
        let persistable = R::Persistable::from(resource);

        let result = persistable.insert(self.db.clone());

        result.map(R::try_from)
            .transpose()
            .unwrap_or_else(|_| panic!("Failed to insert resource into database: {persistable:?}")) //TODO don't unwrap()
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
        let id = id.into_id();

        let result = R::Persistable::get(&id, self.db.clone());

        result.map(R::try_from)
            .transpose()
            .unwrap_or_else(|_| panic!("Failed to get resource from database with id <{id:?}>.")) //TODO don't unwrap()
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
