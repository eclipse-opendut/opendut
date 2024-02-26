
pub mod create {
    use std::collections::HashSet;
    use uuid::Uuid;
    use opendut_carl_api::carl::CarlClient;
    use opendut_types::cluster::{ClusterConfiguration, ClusterId, ClusterName};
    use opendut_types::peer::PeerId;
    use opendut_types::topology::DeviceDescriptor;
    use crate::CreateOutputFormat;

    pub async fn execute(carl: &mut CarlClient, name: String, cluster_id: Option<Uuid>, leader_id: Uuid, device_names: Vec<String>, output: CreateOutputFormat) -> crate::Result<()> {

        let name = ClusterName::try_from(name)
            .map_err(|cause| format!("Error while creating cluster configuration: {}", cause))?;

        let leader = PeerId::from(leader_id); //TODO: check if peer exists
        let cluster_id = ClusterId::from(cluster_id.unwrap_or_else(Uuid::new_v4));

        let all_devices = carl.peers.list_devices().await
            .map_err(|error| format!("Error while listing devices.\n  {}", error))?;
        let checked_devices = check_devices(&all_devices, &device_names);
        let (devices, errors): (Vec<_>, Vec<_>) = checked_devices.into_iter().partition(Result::is_ok);
        let devices = devices.into_iter()
            .map(Result::unwrap)
            .map(|device| device.id)
            .collect::<HashSet<_>>();
        let errors = errors.into_iter().map(Result::unwrap_err).collect::<Vec<_>>();
        if !errors.is_empty() {
            Err(format!("Could not create cluster configuration:\n  {}", errors.join("\n  ")))?
        }

        let configuration = ClusterConfiguration { id: cluster_id, name: Clone::clone(&name), leader, devices };
        carl.cluster.store_cluster_configuration(configuration.clone()).await
            .map_err(|err| format!("Could not store cluster configuration. Make sure the application is running. Error: {}", err))?;

        match output {
            CreateOutputFormat::Text => {
                println!("Successfully stored new cluster configuration.");

                println!("ClusterID: {:?}", cluster_id);
                println!("Name of the Cluster: {:?}", name);
                println!("The following devices are part of the cluster configuration:");
                for device_name in device_names.iter() {
                    println!("\x09{}", device_name);
                };
            }
            CreateOutputFormat::Json => {
                let json = serde_json::to_string(&configuration).unwrap();
                println!("{}", json);
            }
            CreateOutputFormat::PrettyJson => {
                let json = serde_json::to_string_pretty(&configuration).unwrap();
                println!("{}", json);
            }
        }

        Ok(())
    }

    fn check_devices(all_devices: &[DeviceDescriptor], device_names: &[String]) -> Vec<Result<DeviceDescriptor, crate::Error>> {
        let checked_devices = device_names.iter().map(|device_name| {
            let devices = all_devices.iter()
                .filter(|device| device.name.value() == device_name)
                .cloned()
                .collect::<Vec<_>>();

            match devices.as_slice() {
                [] => Err(format!("Device '{}' not found", device_name)),
                [device] => Ok(Clone::clone(device)),
                _ => Err(format!("Multiple devices found for the name '{}'", device_name)),
            }
        }).collect::<Vec<_>>();

        checked_devices
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use googletest::prelude::*;
        use rstest::{fixture, rstest};
        use opendut_types::topology::{DeviceDescription, DeviceId, DeviceName};
        use opendut_types::util::net::NetworkInterfaceName;

        #[fixture]
        fn all_devices() -> Vec<DeviceDescriptor> {
            vec![
                DeviceDescriptor {
                    id: DeviceId::random(),
                    name: DeviceName::try_from("MyDevice").unwrap(),
                    description: DeviceDescription::try_from("").ok(),
                    interface: NetworkInterfaceName::try_from("eth0").unwrap(),
                    tags: vec![],
                },
                DeviceDescriptor {
                    id: DeviceId::random(),
                    name: DeviceName::try_from("YourDevice").unwrap(),
                    description: DeviceDescription::try_from("").ok(),
                    interface: NetworkInterfaceName::try_from("eth0").unwrap(),
                    tags: vec![],
                },
                DeviceDescriptor {
                    id: DeviceId::random(),
                    name: DeviceName::try_from("HisDevice").unwrap(),
                    description: DeviceDescription::try_from("").ok(),
                    interface: NetworkInterfaceName::try_from("eth0").unwrap(),
                    tags: vec![],
                }
            ]
        }

        #[rstest]
        fn test_check_devices(all_devices: Vec<DeviceDescriptor>) {

            let device_names = vec![
                String::from("MyDevice"),
                String::from("HisDevice")
            ];
            assert_that!(check_devices(&all_devices, &device_names),
                unordered_elements_are!(
                    ok(eq(Clone::clone(&all_devices[0]))),
                    ok(eq(Clone::clone(&all_devices[2])))
                )
            );
        }

        #[rstest]
        fn test_that_checked_devices_returns_an_error_for_missing_device(all_devices: Vec<DeviceDescriptor>) {
            let device_names = vec![
                String::from("NoDevice")
            ];
            assert_that!(check_devices(&all_devices, &device_names),
                unordered_elements_are!(
                    err(anything()),
                )
            );
        }

        #[rstest]
        fn test_that_checked_devices_returns_errors_for_missing_devices(all_devices: Vec<DeviceDescriptor>) {
            let device_names = vec![
                String::from("NoDevice"),
                String::from("UnknownDevice")
            ];
            assert_that!(check_devices(&all_devices, &device_names),
                unordered_elements_are!(
                    err(anything()),
                    err(anything()),
                )
            );
        }
    }
}


pub mod list {
    use cli_table::{print_stdout, Table, WithTitle};

    use opendut_carl_api::carl::CarlClient;
    use opendut_types::cluster::{ClusterId, ClusterName};

    use crate::ListOutputFormat;

    #[derive(Table)]
    struct ClusterTable {
        #[table(title = "Name")]
        name: ClusterName,
        #[table(title = "ClusterID")]
        id: ClusterId,
    }

    pub async fn execute(carl: &mut CarlClient, output: ListOutputFormat) -> crate::Result<()> {
        let clusters = carl.cluster.list_cluster_configurations().await
            .map_err(|error| format!("Could not list any cluster configurations.\n  {error}"))?;

            match output {
                ListOutputFormat::Table => {
                    let cluster_table = clusters.into_iter()
                        .map(|cluster| {
                            ClusterTable {
                                name: cluster.name,
                                id: cluster.id,
                            }
                        })
                        .collect::<Vec<_>>();
                    print_stdout(cluster_table.with_title())
                        .expect("List of cluster configurations should be printable as table.");
                }
                ListOutputFormat::Json => {
                    let json = serde_json::to_string(&clusters).unwrap();
                    println!("{}", json);
                }
                ListOutputFormat::PrettyJson => {
                    let json = serde_json::to_string_pretty(&clusters).unwrap();
                    println!("{}", json);
                }
            }
        Ok(())
    }
}

pub mod describe {
    use indoc::indoc;
    use uuid::Uuid;

    use opendut_carl_api::carl::CarlClient;
    use opendut_types::cluster::{ClusterId, ClusterName};
    use opendut_types::peer::{PeerId, PeerName};
    use serde::Serialize;
    use opendut_types::topology::DeviceName;

    use crate::DescribeOutputFormat;

    #[derive(Debug, Serialize)]
    struct ClusterTable {
        name: ClusterName,
        id: ClusterId,
        leader: PeerId,
        peers: Vec<PeerName>,
        devices: Vec<DeviceName>,
    }

    pub async fn execute(carl: &mut CarlClient, id: Uuid, output: DescribeOutputFormat) -> crate::Result<()> {
        let cluster_id = ClusterId::from(id);

        let clusters_configuration = carl.cluster.list_cluster_configurations().await
            .map_err(|_| String::from("Failed to get list of cluster configurations!"))?;

        let cluster_configuration = clusters_configuration.into_iter()
            .find(|cluster_configuration| cluster_configuration.id == cluster_id)
            .ok_or(format!("Failed to find cluster configuration for ClusterID <{}>", cluster_id))?;

        let cluster_devices = {
            let devices = carl.peers.list_devices().await
                .map_err(|_| String::from("Failed to get list of devices!"))?;
            devices.into_iter()
                .filter(|device| cluster_configuration.devices.contains(&device.id))
                .map(|devices| devices.name)
                .collect::<Vec<_>>()
        };

        let cluster_peers = {
            let peers = carl.peers.list_peer_descriptors().await
                .map_err(|_| String::from("Failed to get list of peers!"))?;
            peers.into_iter()
                .filter(|peer| {
                    peer.topology.devices.iter().any(|device| cluster_devices.contains(&device.name))
                })
                .map(|peer| peer.name)
                .collect::<Vec<_>>()
        };

        let table = ClusterTable {
            name: cluster_configuration.name,
            id: cluster_id,
            leader: cluster_configuration.leader,
            peers: cluster_peers,
            devices: cluster_devices,
        };

        let text = match output {
            DescribeOutputFormat::Text => {
                format!(indoc!("
                    Cluster Configuration: {}
                      Id: {}
                      Leader: {}
                      Peers: [{:?}]
                      Devices: [{:?}]
                "), table.name, table.id, table.leader, table.peers, table.devices)
            }
            DescribeOutputFormat::Json => {
                serde_json::to_string(&table).unwrap()
            }
            DescribeOutputFormat::PrettyJson => {
                serde_json::to_string_pretty(&table).unwrap()
            }
        };

        println!("{text}");

        Ok(())
    }
}

pub mod delete {
    use uuid::Uuid;
    use opendut_carl_api::carl::{CarlClient};
    use opendut_types::cluster::ClusterId;

    pub async fn execute(carl: &mut CarlClient, id: Uuid) -> crate::Result<()> {
        let id = ClusterId::from(id);
        let cluster_configuration = carl.cluster.delete_cluster_configuration(id).await
            .map_err(|error| format!("Failed to delete ClusterConfiguration with id <{id}>.\n  {error}"))?;

        println!("Deleted ClusterConfiguration {} <{}> successfully.", cluster_configuration.name, cluster_configuration.id);

        Ok(())
    }
}
