use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;


pub type GlobalResourcesRef = Arc<GlobalResources>;

#[derive(Default)]
pub struct GlobalResources {
    inner: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl GlobalResources {
    pub fn insert<T: Send + Sync + 'static>(&mut self, resource: T) {
        self.inner.insert(resource.type_id(), Box::new(resource));
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> &T {
        let value = self.inner.get(&TypeId::of::<T>())
            .unwrap_or_else(|| panic!("No global Resource found for type {:?}.", type_name::<T>()));

        value
            .downcast_ref::<T>()
            .expect("Failed to downcast global Resource to its type. This should never happen.")
    }

    pub fn complete(self) -> GlobalResourcesRef {
        Arc::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_insert_and_return_a_value() {
        let mut testee = GlobalResources::default();

        let input = TestValue { value: 42 };
        testee.insert(input.clone());

        let result = testee.get::<TestValue>();
        assert_eq!(&input, result);
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestValue { value: u8 }
}
