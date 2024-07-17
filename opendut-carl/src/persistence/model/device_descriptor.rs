use opendut_types::topology::DeviceDescriptor;
use crate::persistence::database::Db;
use crate::persistence::model::{Persistable, PersistableConversionError};

#[derive(Debug)] //diesel::Queryable, diesel::Selectable, diesel::Insertable)]
// #[diesel(table_name = crate::persistence::database::schema::)] //TODO create schema
// #[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistableDeviceDescriptor {
    //TODO
}
impl Persistable<DeviceDescriptor> for PersistableDeviceDescriptor {
    fn insert(&self, db: Db) -> Option<Self> {
        todo!()
    }
}

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
