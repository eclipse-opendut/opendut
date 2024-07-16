use opendut_types::peer::configuration::PeerConfiguration;
use crate::persistence::model::{Persistable, PersistableConversionError};

#[derive(Debug)] //diesel::Queryable, diesel::Selectable, diesel::Insertable)]
// #[diesel(table_name = crate::persistence::database::schema::)] //TODO create schema
// #[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistablePeerConfiguration {
    //TODO
}
impl Persistable<PeerConfiguration> for PersistablePeerConfiguration {
    fn insert(&self) -> Option<Self> {
        todo!()
    }
}

impl From<PeerConfiguration> for PersistablePeerConfiguration {
    fn from(value: PeerConfiguration) -> Self {
        todo!()
    }
}
impl TryFrom<PersistablePeerConfiguration> for PeerConfiguration {
    type Error = PersistableConversionError<PersistablePeerConfiguration, PeerConfiguration>;

    fn try_from(value: PersistablePeerConfiguration) -> Result<Self, Self::Error> {
        todo!()
    }
}
