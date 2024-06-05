use uuid::Uuid;

use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName};
use opendut_types::peer::executor::ExecutorDescriptors;

use crate::persistence::model::{Persistable, PersistableConversionError};

#[derive(diesel::Queryable, diesel::Selectable, diesel::Insertable)]
#[diesel(table_name = crate::persistence::database::schema::peer)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct PersistablePeerDescriptor {
    pub id: Uuid,
    pub name: String,
    pub location: Option<String>,
}
impl Persistable<PeerDescriptor> for PersistablePeerDescriptor {}

impl From<PeerDescriptor> for PersistablePeerDescriptor {
    fn from(value: PeerDescriptor) -> Self {
        Self {
            id: value.id.0,
            name: value.name.value(),
            location: value.location.map(|location| location.value())
        }
    }
}
impl TryFrom<PersistablePeerDescriptor> for PeerDescriptor {
    type Error = PersistableConversionError<PersistablePeerDescriptor, PeerDescriptor>;

    fn try_from(value: PersistablePeerDescriptor) -> Result<Self, Self::Error> {

        let name = PeerName::try_from(value.name)
            .map_err(|cause| PersistableConversionError::new(Box::new(cause)))?;

        let location = value.location
            .map(PeerLocation::try_from)
            .transpose()
            .map_err(|cause| PersistableConversionError::new(Box::new(cause)))?;

        Ok(Self {
            id: PeerId::from(value.id),
            name,
            location,
            network: Default::default(), //TODO
            topology: Default::default(), //TODO
            executors: ExecutorDescriptors { executors: Default::default() }, //TODO
        })
    }
}
