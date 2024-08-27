use diesel::{Connection, PgConnection};
use std::ops::DerefMut;

use opendut_types::peer::{PeerDescriptor, PeerId, PeerNetworkDescriptor};
use opendut_types::topology::Topology;

use super::{query, Persistable};
use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::model::query::peer_descriptor::PersistablePeerDescriptor;
use crate::persistence::model::query::Filter;
use crate::persistence::Storage;

impl Persistable for PeerDescriptor {
    fn insert(self, _peer_id: PeerId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.db.lock().unwrap().transaction::<_, PersistenceError, _>(|connection| {
            insert_into_database(self, connection)
        })
    }

    fn remove(peer_id: PeerId, storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        query::peer_descriptor::remove(peer_id, storage.db.lock().unwrap().deref_mut())
    }

    fn get(peer_id: PeerId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        let result = query::peer_descriptor::list(Filter::By(peer_id), storage.db.lock().unwrap().deref_mut())?
            .first().cloned();
        Ok(result)
    }

    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        query::peer_descriptor::list(Filter::Not, storage.db.lock().unwrap().deref_mut())
    }
}

pub(super) fn insert_into_database(peer_descriptor: PeerDescriptor, connection: &mut PgConnection) -> PersistenceResult<()> {
    let PeerDescriptor { id: peer_id, name, location, network, topology, executors } = peer_descriptor;
    let PeerNetworkDescriptor { interfaces, bridge_name } = network;

    query::peer_descriptor::insert(PersistablePeerDescriptor {
        peer_id: peer_id.uuid,
        name: name.value(),
        location: location.map(|location| location.value()),
        network_bridge_name: bridge_name.map(|name| name.name()),
    }, connection)?;

    for interface in interfaces {
        query::network_interface_descriptor::insert_into_database(interface, peer_id, connection)?;
    }

    let Topology { devices } = topology;

    for device in devices {
        crate::persistence::model::device_descriptor::insert_into_database(device, connection)?;
    }

    for executor in executors.executors {
        query::executor_descriptor::insert_into_database(executor, peer_id, connection)?;
    }

    Ok(())
}
