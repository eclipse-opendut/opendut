use std::sync::{Arc, Mutex};
use diesel::{Connection as _, ConnectionError, PgConnection};
use url::Url;

pub mod schema;

pub type Db = Arc<Mutex<PgConnection>>; //Mutex rather than RwLock, because we share this between threads (i.e. we need it to implement `Sync`)

pub fn connect(url: &Url) -> Result<Db, ConnectionError> {
    let connection = PgConnection::establish(url.as_str())?;
    Ok(Arc::new(Mutex::new(connection)))
}
