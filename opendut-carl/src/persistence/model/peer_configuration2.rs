use opendut_types::peer::configuration::PeerConfiguration2;
use crate::persistence::model::{Persistable, PersistableConversionError};

// #[derive(diesel::Queryable, diesel::Selectable, diesel::Insertable)]
// #[diesel(table_name = crate::persistence::database::schema::)] //TODO create schema
// #[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistablePeerConfiguration2 {
    //TODO
}
impl Persistable<PeerConfiguration2> for PersistablePeerConfiguration2 {}

impl From<PeerConfiguration2> for PersistablePeerConfiguration2 {
    fn from(value: PeerConfiguration2) -> Self {
        todo!()
    }
}
impl TryFrom<PersistablePeerConfiguration2> for PeerConfiguration2 {
    type Error = PersistableConversionError<PersistablePeerConfiguration2, PeerConfiguration2>;

    fn try_from(value: PersistablePeerConfiguration2) -> Result<Self, Self::Error> {
        todo!()
    }
}
