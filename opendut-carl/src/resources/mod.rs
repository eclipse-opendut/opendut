use std::any::{Any, TypeId};
use std::collections::hash_map::{Values, ValuesMut};
use std::collections::HashMap;
use std::marker::PhantomData;

use opendut_types::resources::Id;

pub mod manager;
pub mod ids;

pub trait IntoId<R: Any + Send + Sync> {
    fn into_id(self) -> Id;
}

#[derive(Default)]
pub struct Resources {
    storage: HashMap<TypeId, HashMap<Id, Box<dyn Any + Send + Sync>>>
}

impl Resources {

    pub fn insert<R>(&mut self, id: impl IntoId<R>, resource: R) -> Option<R>
    where R: Any + Send + Sync {
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

    pub fn update<R>(&mut self, id: impl IntoId<R>) -> Update<R>
    where R: Any + Send + Sync {
        let column = self.storage
            .entry(TypeId::of::<R>())
            .or_default();
        Update {
            id: id.into_id(),
            column,
            marker: Default::default(),
        }
    }

    pub fn remove<R>(&mut self, id: impl IntoId<R>) -> Option<R>
    where R: Any + Send + Sync {
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

    pub fn get<R>(&self, id: impl IntoId<R>) -> Option<R>
    where R: Any + Send + Sync + Clone {
        let column = self.column_of::<R>()?;
        column.get(&id.into_id())
            .and_then(|resource| resource
                .downcast_ref()
                .cloned()
            )
    }

    pub fn iter<R>(&self) -> Iter<R>
    where R: Any + Send + Sync {
        Iter::new(self.column_of::<R>().map(HashMap::values))
    }

    pub fn iter_mut<R>(&mut self) -> IterMut<R>
    where R: Any + Send + Sync {
        IterMut::new(self.column_mut_of::<R>().map(HashMap::values_mut))
    }

    pub fn contains<R>(&self, id: impl IntoId<R>) -> bool
    where R: Any + Send + Sync {
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

    pub fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }

    fn column_of<R>(&self) -> Option<&HashMap<Id, Box<dyn Any + Send + Sync>>>
    where R: Any + Send + Sync {
        self.storage.get(&TypeId::of::<R>())
    }

    fn column_mut_of<R>(&mut self) -> Option<&mut HashMap<Id, Box<dyn Any + Send + Sync>>>
        where R: Any + Send + Sync {
        self.storage.get_mut(&TypeId::of::<R>())
    }
}

pub struct Update<'a, R>
where R: Any + Send + Sync {
    id: Id,
    column: &'a mut HashMap<Id, Box<dyn Any + Send + Sync>>,
    marker: PhantomData<R>,
}

impl <R> Update<'_, R>
where R: Any + Send + Sync {

    pub fn modify<F>(self, f: F) -> Self
    where F: FnOnce(&mut R) {
        if let Some(boxed) = self.column.get_mut(&self.id) {
            if let Some(resource) = boxed.downcast_mut() {
                f(resource)
            }
        }
        self
    }

    pub fn or_insert(self, resource: R) {
        self.column.entry(self.id).or_insert_with(|| Box::new(resource));
    }
}

pub struct Iter<'a, R>
where R: Any + Send + Sync {
    column: Option<Values<'a, Id, Box<dyn Any + Send + Sync>>>,
    marker: PhantomData<R>
}

impl <'a, R> Iter<'a, R>
where R: Any + Send + Sync {
    fn new(column: Option<Values<'a, Id, Box<dyn Any + Send + Sync>>>) -> Iter<'a, R> {
        Self {
            column,
            marker: PhantomData
        }
    }
}

impl <'a, R> Iterator for Iter<'a, R>
where R: Any + Send + Sync {

    type Item = &'a R;

    fn next(&mut self) -> Option<Self::Item> {
        let column = self.column.as_mut()?;
        column.next()
            .and_then(|value| value.downcast_ref())
    }
}


pub struct IterMut<'a, R>
where R: Any + Send + Sync {
    column: Option<ValuesMut<'a, Id, Box<dyn Any + Send + Sync>>>,
    marker: PhantomData<R>
}

impl <'a, R> IterMut<'a, R>
where R: Any + Send + Sync {
    fn new(column: Option<ValuesMut<'a, Id, Box<dyn Any + Send + Sync>>>) -> IterMut<'a, R> {
        Self {
            column,
            marker: PhantomData
        }
    }
}

impl <'a, R> Iterator for IterMut<'a, R>
where R: Any + Send + Sync {

    type Item = &'a mut R;

    fn next(&mut self) -> Option<Self::Item> {
        let column = self.column.as_mut()?;
        column.next()
            .and_then(|value| value.downcast_mut())
    }
}
