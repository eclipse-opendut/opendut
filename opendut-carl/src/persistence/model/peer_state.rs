use opendut_types::peer::state::PeerState;
use crate::persistence::model::{Persistable, PersistableConversionError};

#[derive(Debug)] //diesel::Queryable, diesel::Selectable, diesel::Insertable)]
// #[diesel(table_name = crate::persistence::database::schema::)] //TODO create schema
// #[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistablePeerState {
    //TODO
}
impl Persistable<PeerState> for PersistablePeerState {
    fn insert(&self) -> Option<Self> {
        todo!()
    }
}

impl From<PeerState> for PersistablePeerState {
    fn from(value: PeerState) -> Self {
        todo!()
    }
}
impl TryFrom<PersistablePeerState> for PeerState {
    type Error = PersistableConversionError<PersistablePeerState, PeerState>;

    fn try_from(value: PersistablePeerState) -> Result<Self, Self::Error> {
        todo!()
    }
}
