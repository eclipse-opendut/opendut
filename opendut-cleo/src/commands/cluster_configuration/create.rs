use std::collections::HashSet;
use std::ops::Not;

use uuid::Uuid;

use opendut_carl_api::carl::CarlClient;
use opendut_types::cluster::{ClusterConfiguration, ClusterId};
use opendut_types::peer::PeerId;
use opendut_types::topology::{DeviceDescriptor, DeviceId, DeviceName};

use crate::parse::cluster::{ParseableClusterId, ParseableClusterName};
use crate::{ClusterConfigurationDevices, CreateOutputFormat};

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

        let user_specified_devices = {
            let mut devices = select_devices_by_ids(&self.devices.device_ids, &all_devices);

            devices.append(
                &mut select_devices_by_names(&self.devices.device_names, &all_devices)
            );
            devices
        };

        let (devices, errors): (Vec<_>, Vec<_>) = user_specified_devices.into_iter().partition(Result::is_ok);

        {
            let device_errors = errors.into_iter()
                .map(Result::unwrap_err)
                .collect::<Vec<_>>();

            if device_errors.is_empty().not() {
                return Err(format!("Could not create cluster configuration:\n  {}", device_errors.join("\n  ")));
            }
        }

        let devices = devices.into_iter()
            .map(Result::unwrap)
            .collect::<Vec<_>>();

        {
            let duplicates = select_duplicate_devices(&devices);

            if duplicates.is_empty().not() {
                let duplicates = duplicates.iter()
                    .map(|device| format!("{name} <{id}>", name=device.name, id=device.id))
                    .collect::<Vec<_>>()
                    .join(", ");

                return Err(format!("The following devices were specified multiple times: {duplicates}"));
            }
        }

        if devices.len() < 2 {
            return Err("Specify at least 2 devices per cluster configuration.".to_string());
        }

        let device_ids = devices.clone().into_iter()
            .map(|device| device.id)
            .collect::<HashSet<_>>();

        let configuration = ClusterConfiguration { 
            id: cluster_id, 
            name: Clone::clone(&cluster_name), 
            leader, 
            devices: device_ids 
        };
        
        create_cluster_configuration(configuration, carl, &output).await?;

        Ok(())
    }
}

pub async fn create_cluster_configuration(configuration: ClusterConfiguration, carl: &mut CarlClient, output: &CreateOutputFormat) -> crate::Result<()> {
    carl.cluster.store_cluster_configuration(configuration.clone()).await
        .map_err(|err| format!("Could not store cluster configuration. Make sure the application is running. Error: {}", err))?;
    
    let devices = carl.peers.list_devices().await.map_err(|error| format!("Error while trying to match devices.\n  {}", error))?;
    let device_names = devices.clone().into_iter()
        .map(|device| device.name)
        .collect::<Vec<_>>();

    match output {
        CreateOutputFormat::Text => {
            println!("Successfully stored new cluster configuration.");
        
            println!("ClusterID: {}", configuration.id);
            println!("Name of the Cluster: {}", configuration.name);
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


fn select_duplicate_devices(devices: &[DeviceDescriptor]) -> Vec<DeviceDescriptor> {
    let mut devices = devices.to_vec();

    devices.sort_by_key(|device| device.id.0);

    devices.chunk_by(|a, b| a == b)
        .filter(|chunk| chunk.len() > 1)
        .map(|chunk| chunk.first().unwrap())
        .cloned()
        .collect::<Vec<_>>()
}

fn select_devices_by_ids(device_ids: &[DeviceId], devices_list: &[DeviceDescriptor]) -> Vec<Result<DeviceDescriptor, crate::Error>> {
    device_ids.iter()
        .map(|device_id| {
            devices_list.iter()
                .find(|device| &device.id == device_id)
                .cloned()
                .ok_or(format!("Device '{}' not found", device_id))
        })
        .collect()
}

fn select_devices_by_names(device_names: &[DeviceName], devices_list: &[DeviceDescriptor]) -> Vec<Result<DeviceDescriptor, crate::Error>> {
    device_names.iter()
        .map(|device_name| {
            let devices = devices_list.iter()
                .filter(|device| &device.name == device_name)
                .cloned()
                .collect::<Vec<_>>();

            match devices.as_slice() {
                [] => Err(format!("Device '{}' not found", device_name)),
                [device] => Ok(Clone::clone(device)),
                _ => Err(format!("Multiple devices found for the name '{}'", device_name)),
            }
        })
        .collect()
}


#[cfg(test)]
mod test {
    use googletest::prelude::*;

    use opendut_types::topology::{DeviceDescription, DeviceId, DeviceName};
    use opendut_types::util::net::NetworkInterfaceId;

    use super::*;

    #[test]
    fn should_select_devices_from_ids() -> anyhow::Result<()> {
        let all_devices = all_devices();

        let device_ids = vec![
            all_devices[2].id,
            all_devices[0].id
        ];
        assert_that!(
            select_devices_by_ids(&device_ids, &all_devices),
            unordered_elements_are!(
                ok(eq(&all_devices[0])),
                ok(eq(&all_devices[2]))
            )
        );

        Ok(())
    }

    #[test]
    fn should_select_devices_from_names() -> anyhow::Result<()> {
        let all_devices = all_devices();

        let device_names = vec![
            DeviceName::try_from("MyDevice")?,
            DeviceName::try_from("HisDevice")?,
        ];
        assert_that!(
            select_devices_by_names(&device_names, &all_devices),
            unordered_elements_are!(
                ok(eq(&all_devices[0])),
                ok(eq(&all_devices[2]))
            )
        );

        Ok(())
    }

    #[test]
    fn should_return_an_error_for_missing_device_when_selecting_by_names() -> anyhow::Result<()> {
        let all_devices = all_devices();

        let device_names = vec![
            DeviceName::try_from("NoDevice")?,
        ];
        assert_that!(
            select_devices_by_names(&device_names, &all_devices),
            unordered_elements_are!(
                err(anything()),
            )
        );
        Ok(())
    }

    #[test]
    fn should_return_an_error_for_missing_device_when_selecting_by_ids() -> anyhow::Result<()> {
        let all_devices = all_devices();

        let device_ids = vec![
            DeviceId::random(),
        ];
        assert_that!(
            select_devices_by_ids(&device_ids, &all_devices),
            unordered_elements_are!(
                err(anything()),
            )
        );
        Ok(())
    }

    #[test]
    fn should_select_duplicates() -> anyhow::Result<()> {
        let all_devices = all_devices();

        let devices_with_duplicates = vec![
            all_devices[0].clone(),
            all_devices[1].clone(),
            all_devices[0].clone(),
        ];

        assert_that!(
            select_duplicate_devices(&devices_with_duplicates),
            unordered_elements_are!(
                eq(&all_devices[0])
            )
        );

        Ok(())
    }

    fn all_devices() -> Vec<DeviceDescriptor> {
        vec![
            DeviceDescriptor {
                id: DeviceId::random(),
                name: DeviceName::try_from("MyDevice").unwrap(),
                description: DeviceDescription::try_from("").ok(),
                interface: NetworkInterfaceId::random(),
                tags: vec![],
            },
            DeviceDescriptor {
                id: DeviceId::random(),
                name: DeviceName::try_from("YourDevice").unwrap(),
                description: DeviceDescription::try_from("").ok(),
                interface: NetworkInterfaceId::random(),
                tags: vec![],
            },
            DeviceDescriptor {
                id: DeviceId::random(),
                name: DeviceName::try_from("HisDevice").unwrap(),
                description: DeviceDescription::try_from("").ok(),
                interface: NetworkInterfaceId::random(),
                tags: vec![],
            }
        ]
    }
}
