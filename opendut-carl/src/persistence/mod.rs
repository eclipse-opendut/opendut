use std::sync::Mutex;

use diesel::PgConnection;

use crate::resources::storage::volatile::VolatileResourcesStorage;

pub mod database;
pub mod model;

pub struct Storage {
    pub db: Db,
    pub memory: Memory,
}
pub type Db = Mutex<PgConnection>; //Mutex rather than RwLock, because we share this between threads (i.e. we need it to implement `Sync`)
pub type Memory = VolatileResourcesStorage;
