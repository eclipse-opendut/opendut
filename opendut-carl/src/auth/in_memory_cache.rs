use std::collections::HashMap;
use std::hash::Hash;
use std::sync::RwLock;

#[derive(thiserror::Error, Debug)]
pub enum CacheError {
    #[error("Could not acquire Write Lock: {0}")]
    WriteLockFailed(String),
    #[error("Could not acquire Read Lock: {0}")]
    ReadLockFailed(String),
}

#[derive(Debug, Default)]
pub struct CustomInMemoryCache<K, V> {
    cache: RwLock<HashMap<K, V>>
}

impl<K, V> Clone for CustomInMemoryCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        match self.cache.read() {
            Ok(read_lock) => {
                CustomInMemoryCache {
                    cache: RwLock::new(read_lock.clone())
                }
            },
            Err(error) => {
                panic!("Could not acquire Read Lock: {error}");
            }
        }
    }
}

impl<K, V> CustomInMemoryCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Creates a new cache.
    pub fn new() -> Self {
        CustomInMemoryCache {
            cache: RwLock::new(HashMap::new())
        }
    }

    /// Inserts a key-value pair into the cache.
    pub fn insert(&mut self, key: K, value: V) -> Result<(), CacheError> {
        match self.cache.write() {
            Ok(mut write_lock) => {
                write_lock.insert(key, value);
                Ok(())
            }
            Err(error) => {
                Err(CacheError::WriteLockFailed(error.to_string()))
            }
        }
    }

    /// Retrieves a value by key.
    pub fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        match self.cache.read() {
            Ok(read_lock) => {
                Ok(read_lock.get(key).cloned())
            }
            Err(error) => {
                Err(CacheError::ReadLockFailed(error.to_string()))
            }
        }
    }

    pub fn delete_all(&mut self) -> Result<(), CacheError>{
        match self.cache.write() {
            Ok(mut write_lock) => {
                write_lock.clear();
                Ok(())
            }
            Err(error) => {
                Err(CacheError::WriteLockFailed(error.to_string()))
            }
        }
    }

}
