use opendut_types::resources::Id;
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

    fn get(id: &Id, db: Db) -> Option<Self> {
        todo!()
    }
}

impl From<DeviceDescriptor> for PersistableDeviceDescriptor {
    fn from(value: DeviceDescriptor) -> Self {
        todo!()
    }
}
impl TryInto<DeviceDescriptor> for PersistableDeviceDescriptor {
    type Error = PersistableConversionError<PersistableDeviceDescriptor, DeviceDescriptor>;

    fn try_into(self) -> Result<DeviceDescriptor, Self::Error> {
        todo!()
    }
}
