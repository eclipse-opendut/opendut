use diesel::{Connection, PgConnection};
use opendut_types::topology::{DeviceDescriptor, DeviceId};
use std::ops::DerefMut;

use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::model::query::device_descriptor::PersistableDeviceDescriptor;
use crate::persistence::model::query::device_tag::PersistableDeviceTag;
use crate::persistence::model::query::Filter;
use crate::persistence::model::{query, Persistable};
use crate::persistence::Storage;

impl Persistable for DeviceDescriptor {
    fn insert(self, _device_id: DeviceId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.db.lock().unwrap().transaction::<_, PersistenceError, _>(|connection| {
            insert_into_database(self, connection)
        })
    }

    fn remove(_device_id: DeviceId, _storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        todo!("implement removal of device_descriptors from database")
    }

    fn get(device_id: DeviceId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        let result = query::device_descriptor::list(Filter::By(device_id), storage.db.lock().unwrap().deref_mut())?
            .first().cloned();
        Ok(result)
    }

    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        query::device_descriptor::list(Filter::Not, storage.db.lock().unwrap().deref_mut())
    }
}

pub(super) fn insert_into_database(device_descriptor: DeviceDescriptor, connection: &mut PgConnection) -> PersistenceResult<()> {
    let DeviceDescriptor { id, name, description, interface, tags } = device_descriptor;

    let name = name.value().to_owned();
    let description = description.map(|description| description.value().to_owned());
    let network_interface_id = Some(interface.uuid);

    query::device_descriptor::insert(PersistableDeviceDescriptor {
        device_id: id.0,
        name,
        description,
        network_interface_id,
    }, connection)?;

    for tag in tags {
        query::device_tag::insert(PersistableDeviceTag {
            device_id: id.0,
            name: tag.value().to_owned(),
        }, connection)?;
    }

    Ok(())
}
