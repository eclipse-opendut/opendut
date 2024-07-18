use opendut_types::peer::configuration::PeerConfiguration;
use opendut_types::resources::Id;
use crate::persistence::database::Db;
use crate::persistence::model::{Persistable, PersistableConversionError};

#[derive(Debug)] //diesel::Queryable, diesel::Selectable, diesel::Insertable)]
// #[diesel(table_name = crate::persistence::database::schema::)] //TODO create schema
// #[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistablePeerConfiguration {
    //TODO
}
impl Persistable<PeerConfiguration> for PersistablePeerConfiguration {
    fn insert(&self, db: Db) -> Option<Self> {
        todo!()
    }

    fn get(id: &Id, db: Db) -> Option<Self> {
        todo!()
    }
}

impl From<PeerConfiguration> for PersistablePeerConfiguration {
    fn from(value: PeerConfiguration) -> Self {
        todo!()
    }
}
impl TryInto<PeerConfiguration> for PersistablePeerConfiguration {
    type Error = PersistableConversionError<PersistablePeerConfiguration, PeerConfiguration>;

    fn try_into(self) -> Result<PeerConfiguration, Self::Error> {
        todo!()
    }
}
