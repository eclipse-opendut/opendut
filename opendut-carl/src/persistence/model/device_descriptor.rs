use opendut_types::topology::DeviceDescriptor;
use crate::persistence::model::{Persistable, PersistableConversionError};

// #[derive(diesel::Queryable, diesel::Selectable, diesel::Insertable)]
// #[diesel(table_name = crate::persistence::database::schema::)] //TODO create schema
// #[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistableDeviceDescriptor {
    //TODO
}
impl Persistable<DeviceDescriptor> for PersistableDeviceDescriptor {}

impl From<DeviceDescriptor> for PersistableDeviceDescriptor {
    fn from(value: DeviceDescriptor) -> Self {
        todo!()
    }
}
impl TryFrom<PersistableDeviceDescriptor> for DeviceDescriptor {
    type Error = PersistableConversionError<PersistableDeviceDescriptor, DeviceDescriptor>;

    fn try_from(value: PersistableDeviceDescriptor) -> Result<Self, Self::Error> {
        todo!()
    }
}
