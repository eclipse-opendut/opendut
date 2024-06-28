use std::collections::HashSet;
use std::ops::Not;

use uuid::Uuid;

use opendut_carl_api::carl::CarlClient;
use opendut_types::cluster::{ClusterConfiguration, ClusterId};
use opendut_types::peer::PeerId;
use opendut_types::topology::{DeviceDescriptor, DeviceName};

use crate::{ClusterConfigurationDevices, CreateOutputFormat};
use crate::parse::cluster::{ParseableClusterId, ParseableClusterName};

/// Create a cluster configuration
#[derive(clap::Parser)]
pub struct CreateClusterConfigurationCli {
    ///Name of the cluster
    #[arg(short, long)]
    name: ParseableClusterName,
    ///ClusterID
    #[arg(short, long)]
    cluster_id: Option<ParseableClusterId>,
    ///PeerID of the leader
    #[arg(short, long)]
    leader_id: Uuid,
    ///List of devices in cluster
    #[clap(flatten)]
    devices: ClusterConfigurationDevices,
}

impl CreateClusterConfigurationCli {
    pub async fn execute(
        self,
        carl: &mut CarlClient,
        output: CreateOutputFormat,
    ) -> crate::Result<()> {
        let ParseableClusterName(cluster_name) = self.name;

        let cluster_id = self.cluster_id
            .map(|ParseableClusterId(id)| id)
            .unwrap_or_else(ClusterId::random);
        
        let cluster_deployments = carl.cluster.list_cluster_deployments().await
            .map_err(|_| String::from("Failed to get list of cluster deployments!"))?;
        if cluster_deployments.into_iter().any(|cluster_deployment| cluster_deployment.id == cluster_id) {
            Err(format!("Cluster <{}> can not be updated while it is deployed.", cluster_id))?
        };

        let leader = PeerId::from(self.leader_id); //TODO: check if peer exists

        let all_devices = carl.peers.list_devices().await
            .map_err(|error| format!("Error while listing devices.\n  {}", error))?;
        let checked_devices = check_devices(&all_devices, &self.devices.device_names, &self.devices.device_ids);
        let (devices, errors): (Vec<_>, Vec<_>) = checked_devices.into_iter().partition(Result::is_ok);
        let devices = devices.into_iter()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        let device_names = devices.clone().into_iter()
            .map(|device| device.name)
            .collect::<Vec<_>>();
        let device_ids = devices.clone().into_iter()
            .map(|device| device.id)
            .collect::<HashSet<_>>();
        let errors = errors.into_iter().map(Result::unwrap_err).collect::<Vec<_>>();
        if !errors.is_empty() {
            Err(format!("Could not create cluster configuration:\n  {}", errors.join("\n  ")))?
        }
        if devices.len() < 2 {
            Err("Specify at least 2 devices per cluster configuration.".to_string())?
        }

        let configuration = ClusterConfiguration { id: cluster_id, name: Clone::clone(&cluster_name), leader, devices: device_ids };
        carl.cluster.store_cluster_configuration(configuration.clone()).await
            .map_err(|err| format!("Could not store cluster configuration. Make sure the application is running. Error: {}", err))?;

        match output {
            CreateOutputFormat::Text => {
                println!("Successfully stored new cluster configuration.");

                println!("ClusterID: {}", cluster_id);
                println!("Name of the Cluster: {}", cluster_name);
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
}

fn check_devices(all_devices: &[DeviceDescriptor], device_names: &[DeviceName], device_ids: &[String]) -> Vec<Result<DeviceDescriptor, crate::Error>> {
    let mut checked_devices_ids = device_ids.iter().map(|device_id| {
        let maybe_device = all_devices.iter().find(|device| device.id.to_string().starts_with(device_id));
        if let Some(device) = maybe_device {
            Ok(Clone::clone(device))
        }
        else {
            Err(format!("Device '{}' not found", device_id))
        }
    }).collect::<Vec<_>>();

    let already_checked_device_names = checked_devices_ids.clone().iter_mut()
        .filter_map(|device| match device {
            Ok(device) => Some(Clone::clone(&device.name)),
            Err(_) => None,
        })
        .collect::<Vec<_>>();

    let remaining_device_names = device_names.iter()
        .filter(|device_name| already_checked_device_names.contains(device_name).not())
        .collect::<Vec<_>>();

    let mut checked_devices_names = remaining_device_names.iter().map(|&device_name| {
        let devices = all_devices.iter()
            .filter(|device| &device.name == device_name)
            .cloned()
            .collect::<Vec<_>>();

        match devices.as_slice() {
            [] => Err(format!("Device '{}' not found", device_name)),
            [device] => Ok(Clone::clone(device)),
            _ => Err(format!("Multiple devices found for the name '{}'", device_name)),
        }
    }).collect::<Vec<_>>();

    checked_devices_names.append(&mut checked_devices_ids);
    checked_devices_names
}

#[cfg(test)]
mod test {
    use googletest::prelude::*;
    use rstest::{fixture, rstest};

    use opendut_types::topology::{DeviceDescription, DeviceId, DeviceName};
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceName};

    use super::*;

    #[fixture]
    fn all_devices() -> Vec<DeviceDescriptor> {
        vec![
            DeviceDescriptor {
                id: DeviceId::random(),
                name: DeviceName::try_from("MyDevice").unwrap(),
                description: DeviceDescription::try_from("").ok(),
                interface: NetworkInterfaceDescriptor {
                    name: NetworkInterfaceName::try_from("eth0").unwrap(),
                    configuration: NetworkInterfaceConfiguration::Ethernet,
                },
                tags: vec![],
            },
            DeviceDescriptor {
                id: DeviceId::random(),
                name: DeviceName::try_from("YourDevice").unwrap(),
                description: DeviceDescription::try_from("").ok(),
                interface: NetworkInterfaceDescriptor {
                    name: NetworkInterfaceName::try_from("eth0").unwrap(),
                    configuration: NetworkInterfaceConfiguration::Ethernet,
                },
                tags: vec![],
            },
            DeviceDescriptor {
                id: DeviceId::random(),
                name: DeviceName::try_from("HisDevice").unwrap(),
                description: DeviceDescription::try_from("").ok(),
                interface: NetworkInterfaceDescriptor {
                    name: NetworkInterfaceName::try_from("eth0").unwrap(),
                    configuration: NetworkInterfaceConfiguration::Ethernet,
                },
                tags: vec![],
            }
        ]
    }

    #[rstest]
    fn test_check_devices_with_names(all_devices: Vec<DeviceDescriptor>) -> anyhow::Result<()> {

        let device_names = vec![
            DeviceName::try_from("MyDevice")?,
            DeviceName::try_from("HisDevice")?,
        ];
        assert_that!(check_devices(&all_devices, &device_names, &[]),
            unordered_elements_are!(
                ok(eq(Clone::clone(&all_devices[0]))),
                ok(eq(Clone::clone(&all_devices[2])))
            )
        );
        Ok(())
    }

    #[rstest]
    fn test_check_devices_with_ids(all_devices: Vec<DeviceDescriptor>) {

        let device_ids = vec![
            all_devices[2].id.to_string(),
            all_devices[0].id.to_string()
        ];
        assert_that!(check_devices(&all_devices, &[], &device_ids),
            unordered_elements_are!(
                ok(eq(Clone::clone(&all_devices[0]))),
                ok(eq(Clone::clone(&all_devices[2])))
            )
        );
    }

    #[rstest]
    fn test_that_checked_devices_returns_an_error_for_missing_device(all_devices: Vec<DeviceDescriptor>) -> anyhow::Result<()> {
        let device_names = vec![
            DeviceName::try_from("NoDevice")?,
        ];
        assert_that!(check_devices(&all_devices, &device_names, &[]),
            unordered_elements_are!(
                err(anything()),
            )
        );
        Ok(())
    }

    #[rstest]
    fn test_that_checked_devices_returns_errors_for_missing_devices(all_devices: Vec<DeviceDescriptor>) -> anyhow::Result<()> {
        let device_names = vec![
            DeviceName::try_from("NoDevice")?,
            DeviceName::try_from("UnknownDevice")?,
        ];
        let device_ids = vec![
            DeviceId::random().to_string(),
        ];
        assert_that!(check_devices(&all_devices, &device_names, &device_ids),
            unordered_elements_are!(
                err(anything()),
                err(anything()),
                err(anything()),
            )
        );
        Ok(())
    }

    #[rstest]
    fn test_that_checked_devices_adds_devices_defined_multiple_times_only_once(all_devices: Vec<DeviceDescriptor>) -> anyhow::Result<()> {
        let device_names = vec![
            DeviceName::try_from("MyDevice")?,
            DeviceName::try_from("HisDevice")?,
        ];
        let device_ids = vec![
            all_devices[0].id.to_string()
        ];
        assert_that!(check_devices(&all_devices, &device_names, &device_ids),
            unordered_elements_are!(
                ok(eq(Clone::clone(&all_devices[0]))),
                ok(eq(Clone::clone(&all_devices[2])))
            )
        );
        Ok(())
    }
}
