use std::ops::DerefMut;

use diesel::Connection;

use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
use opendut_types::peer::executor::ExecutorDescriptors;
use opendut_types::util::net::NetworkInterfaceConfiguration;

use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::model::persistable::network_interface_descriptor::PersistableNetworkInterfaceDescriptor;
use crate::persistence::model::persistable::network_interface_kind::PersistableNetworkInterfaceKind;
use crate::persistence::model::persistable::peer_descriptor::PersistablePeerDescriptor;
use crate::persistence::Storage;

use super::{Persistable, PersistableConversionError};

impl Persistable for PeerDescriptor {
    fn insert(self, peer_id: PeerId, storage: &mut Storage) -> PersistenceResult<()> {
        let PeerDescriptor { id: peer_id, name, location, network, topology, executors } = self;
        let PeerNetworkDescriptor { interfaces, bridge_name } = network;

        storage.db.lock().unwrap().deref_mut().transaction::<_, PersistenceError, _>(|connection| {

            PersistablePeerDescriptor {
                peer_id: peer_id.uuid,
                name: name.value(),
                location: location.map(|location| location.value()),
                network_bridge_name: bridge_name.map(|name| name.name()),
            }.insert(peer_id, storage.db.lock().unwrap().deref_mut())?;

            for interface in interfaces {
                let (kind, maybe_configuration) = match interface.configuration {
                    NetworkInterfaceConfiguration::Ethernet => {
                        (PersistableNetworkInterfaceKind::Ethernet, None)
                    }
                    NetworkInterfaceConfiguration::Can { bitrate, sample_point, fd, data_bitrate, data_sample_point } => {
                        (PersistableNetworkInterfaceKind::Can, Some("TODO configuration")) //TODO configuration
                    }
                };
                PersistableNetworkInterfaceDescriptor {
                    network_interface_id: interface.id.uuid,
                    name: interface.name.name(),
                    kind,
                    peer_id: peer_id.uuid,
                }.insert(interface.id, connection)?;
            }

            // TODO persist other fields

            Ok(())
        })
    }

    fn get(peer_id: PeerId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        let persistable = PersistablePeerDescriptor::get(peer_id, storage.db.lock().unwrap().deref_mut())?;

        let result = persistable
            .map(TryInto::try_into)
            .transpose()
            .map_err(|cause| PersistenceError::get::<Self>(peer_id.uuid, cause).context("Failed to convert from PersistablePeerDescriptor."))?;
        Ok(result)
    }

    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        let persistables = PersistablePeerDescriptor::list(storage.db.lock().unwrap().deref_mut())?;

        let result = persistables
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|cause| PersistenceError::list::<Self>(cause).context("Failed to convert from list of PersistablePeerDescriptors."))?;
        Ok(result)
    }
}


impl TryInto<PeerDescriptor> for PersistablePeerDescriptor {
    type Error = PersistableConversionError<PersistablePeerDescriptor, PeerDescriptor>;

    fn try_into(self) -> Result<PeerDescriptor, Self::Error> {

        let name = PeerName::try_from(self.name)
            .map_err(|cause| PersistableConversionError::new(Box::new(cause)))?;

        let location = self.location
            .map(PeerLocation::try_from)
            .transpose()
            .map_err(|cause| PersistableConversionError::new(Box::new(cause)))?;

        Ok(PeerDescriptor {
            id: PeerId::from(self.peer_id),
            name,
            location,
            network: Default::default(), //TODO
            topology: Default::default(), //TODO
            executors: ExecutorDescriptors { executors: Default::default() }, //TODO
        })
    }
}
