use opendut_types::peer::configuration::PeerConfiguration;
use crate::persistence::model::{Persistable, PersistableConversionError};

// #[derive(diesel::Queryable, diesel::Selectable, diesel::Insertable)]
// #[diesel(table_name = crate::persistence::database::schema::)] //TODO create schema
// #[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistablePeerConfiguration {
    //TODO
}
impl Persistable<PeerConfiguration> for PersistablePeerConfiguration {}

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
