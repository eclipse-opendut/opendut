use opendut_types::peer::configuration::PeerConfiguration2;
use opendut_types::resources::Id;
use crate::persistence::database::Db;
use crate::persistence::model::{Persistable, PersistableConversionError};

#[derive(Debug)] //diesel::Queryable, diesel::Selectable, diesel::Insertable)]
// #[diesel(table_name = crate::persistence::database::schema::)] //TODO create schema
// #[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistablePeerConfiguration2 {
    //TODO
}
impl Persistable<PeerConfiguration2> for PersistablePeerConfiguration2 {
    fn insert(&self, db: Db) -> Option<Self> {
        todo!()
    }

    fn get(id: &Id, db: Db) -> Option<Self> {
        todo!()
    }
}

impl From<PeerConfiguration2> for PersistablePeerConfiguration2 {
    fn from(value: PeerConfiguration2) -> Self {
        todo!()
    }
}
impl TryInto<PeerConfiguration2> for PersistablePeerConfiguration2 {
    type Error = PersistableConversionError<PersistablePeerConfiguration2, PeerConfiguration2>;

    fn try_into(self) -> Result<PeerConfiguration2, Self::Error> {
        todo!()
    }
}
