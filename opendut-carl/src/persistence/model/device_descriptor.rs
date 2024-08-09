use std::ops::DerefMut;
use diesel::{Connection, PgConnection};
use opendut_types::peer::PeerId;
use opendut_types::topology::{DeviceDescriptor, DeviceId};

use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::model::{Persistable, persistable};
use crate::persistence::model::persistable::device_descriptor::PersistableDeviceDescriptor;
use crate::persistence::model::persistable::device_tag::PersistableDeviceTag;
use crate::persistence::Storage;

impl Persistable for DeviceDescriptor {
    fn insert(self, _device_id: DeviceId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.db.lock().unwrap().deref_mut().transaction::<_, PersistenceError, _>(|connection| {
            insert_into_database(self, connection)
        })
    }

    fn remove(_device_id: DeviceId, _storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        todo!("implement removal of device_descriptors from database")
    }

    fn get(device_id: DeviceId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        get_from_database(device_id, storage.db.lock().unwrap().deref_mut())
    }

    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        list_database(storage.db.lock().unwrap().deref_mut())
    }
}

pub(super) fn insert_into_database(device_descriptor: DeviceDescriptor, connection: &mut PgConnection) -> PersistenceResult<()> {
    let DeviceDescriptor { id, name, description, interface, tags } = device_descriptor;

    let name = name.value().to_owned();
    let description = description.map(|description| description.value().to_owned());
    let network_interface_id = Some(interface.uuid);

    PersistableDeviceDescriptor {
        device_id: id.0,
        name,
        description,
        network_interface_id,
    }.insert(id, connection)?;

    for tag in tags {
        PersistableDeviceTag {
            device_id: id.0,
            name: tag.value().to_owned(),
        }.insert(connection)?;
    }

    Ok(())
}
pub(super) fn get_from_database(device_id: DeviceId, connection: &mut PgConnection) -> PersistenceResult<Option<DeviceDescriptor>> {
    persistable::device_descriptor::get(device_id, connection)
}
pub(super) fn list_database(connection: &mut PgConnection) -> PersistenceResult<Vec<DeviceDescriptor>> {
    persistable::device_descriptor::list(connection)
}
pub(super) fn list_database_filtered_by_peer_id(peer_id: PeerId, connection: &mut PgConnection) -> PersistenceResult<Vec<DeviceDescriptor>> {
    persistable::device_descriptor::list_filtered_by_peer(peer_id, connection)
}
