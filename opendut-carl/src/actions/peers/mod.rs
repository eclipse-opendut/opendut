pub mod assign_cluster;
pub mod delete_peer_descriptor;
pub mod generate_cleo_setup;
pub mod generate_peer_setup;
pub mod get_peer_state;
pub mod list_devices;
pub mod list_peer_descriptors;
pub mod store_peer_descriptor;
pub mod unassign_cluster;

#[cfg(test)]
mod testing {
    use rstest::*;

    use opendut_types::peer::executor::ExecutorDescriptors;
    use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
    use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, Topology};
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

    use crate::actions::StorePeerDescriptorOptions;
    use crate::vpn::Vpn;

    pub struct Fixture {
        pub vpn: Vpn,
        pub peer_a_id: PeerId,
        pub peer_a_descriptor: PeerDescriptor,
        pub peer_a_device_1: DeviceId,
        pub peer_a_device_2: DeviceId,
    }

    #[fixture]
    pub fn fixture() -> Fixture {
        let peer_a_id = PeerId::random();
        let peer_a_network_interface_1 = NetworkInterfaceId::random();
        let peer_a_network_interface_2 = NetworkInterfaceId::random();
        let peer_a_device_1 = DeviceId::random();
        let peer_a_device_2 = DeviceId::random();
        let peer_a_descriptor = PeerDescriptor {
            id: peer_a_id,
            name: PeerName::try_from("PeerA").unwrap(),
            location: PeerLocation::try_from("Ulm").ok(),
            network: PeerNetworkDescriptor {
                interfaces: vec![
                    NetworkInterfaceDescriptor {
                        id: peer_a_network_interface_1,
                        name: NetworkInterfaceName::try_from("eth0").unwrap(),
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    },
                    NetworkInterfaceDescriptor {
                        id: peer_a_network_interface_2,
                        name: NetworkInterfaceName::try_from("eth1").unwrap(),
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    },
                ],
                bridge_name: Some(NetworkInterfaceName::try_from("br-opendut-1").unwrap()),
            },
            topology: Topology {
                devices: vec![
                    DeviceDescriptor {
                        id: peer_a_device_1,
                        name: DeviceName::try_from("PeerA_Device_1").unwrap(),
                        description: DeviceDescription::try_from("Huii").ok(),
                        interface: peer_a_network_interface_1,
                        tags: vec![],
                    },
                    DeviceDescriptor {
                        id: peer_a_device_2,
                        name: DeviceName::try_from("PeerA_Device_2").unwrap(),
                        description: DeviceDescription::try_from("Huii").ok(),
                        interface: peer_a_network_interface_2,
                        tags: vec![],
                    }
                ]
            },
            executors: ExecutorDescriptors {
                executors: vec![],
            }
        };
        Fixture {
            vpn: Vpn::Disabled,
            peer_a_id,
            peer_a_descriptor,
            peer_a_device_1,
            peer_a_device_2,
        }
    }

    #[fixture]
    pub fn store_peer_descriptor_options() -> StorePeerDescriptorOptions {
        StorePeerDescriptorOptions {
            bridge_name_default: NetworkInterfaceName::try_from("br-opendut").unwrap(),
        }
    }
}
