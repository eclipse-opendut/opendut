mod clusters;
pub use clusters::create_cluster_configuration::*;
pub use clusters::delete_cluster_configuration::*;
pub use clusters::delete_cluster_deployment::*;
pub use clusters::determine_cluster_peer_states::*;
pub use clusters::determine_cluster_peers::*;
pub use clusters::list_deployed_clusters::*;
pub use clusters::store_cluster_deployment::*;

mod peers;
pub use peers::assign_cluster::*;
pub use peers::delete_peer_descriptor::*;
pub use peers::generate_cleo_setup::*;
pub use peers::generate_peer_setup::*;
pub use peers::get_peer_state::*;
pub use peers::list_devices::*;
pub use peers::list_peer_descriptors::*;
pub use peers::list_peer_member_states::*;
pub use peers::list_peer_states::*;
pub use peers::store_peer_descriptor::*;


#[cfg(test)]
mod testing {
    use opendut_types::peer::executor::ExecutorDescriptors;
    use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
    use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, Topology};
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

    pub struct PeerFixture {
        pub id: PeerId,
        pub descriptor: PeerDescriptor,
        pub device_1: DeviceId,
        pub device_2: DeviceId,
    }
    impl PeerFixture {
        pub fn new() -> PeerFixture {
            let id = PeerId::random();
            let network_interface_1 = NetworkInterfaceId::random();
            let network_interface_2 = NetworkInterfaceId::random();
            let device_1 = DeviceId::random();
            let device_2 = DeviceId::random();

            let descriptor = PeerDescriptor {
                id,
                name: PeerName::try_from("PeerA").unwrap(),
                location: PeerLocation::try_from("Ulm").ok(),
                network: PeerNetworkDescriptor {
                    interfaces: vec![
                        NetworkInterfaceDescriptor {
                            id: network_interface_1,
                            name: NetworkInterfaceName::try_from("eth0").unwrap(),
                            configuration: NetworkInterfaceConfiguration::Ethernet,
                        },
                        NetworkInterfaceDescriptor {
                            id: network_interface_2,
                            name: NetworkInterfaceName::try_from("eth1").unwrap(),
                            configuration: NetworkInterfaceConfiguration::Ethernet,
                        },
                    ],
                    bridge_name: Some(NetworkInterfaceName::try_from("br-opendut-1").unwrap()),
                },
                topology: Topology {
                    devices: vec![
                        DeviceDescriptor {
                            id: device_1,
                            name: DeviceName::try_from("PeerA_Device_1").unwrap(),
                            description: DeviceDescription::try_from("Huii").ok(),
                            interface: network_interface_1,
                            tags: vec![],
                        },
                        DeviceDescriptor {
                            id: device_2,
                            name: DeviceName::try_from("PeerA_Device_2").unwrap(),
                            description: DeviceDescription::try_from("Huii").ok(),
                            interface: network_interface_2,
                            tags: vec![],
                        }
                    ]
                },
                executors: ExecutorDescriptors {
                    executors: vec![],
                }
            };
            PeerFixture {
                id,
                descriptor,
                device_1,
                device_2,
            }
        }
    }
}
