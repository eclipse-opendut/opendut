use uuid::Uuid;

use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::PeerId;
use opendut_types::util::net::{CanSamplePoint, NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceName};

use crate::{CreateOutputFormat, DescribeOutputFormat, NetworkInterfaceType};

pub async fn execute(
    carl: &mut CarlClient,
    peer_id: Uuid,
    interface_type: NetworkInterfaceType,
    interface_name: String,
    output: CreateOutputFormat,
) -> crate::Result<()> {
    let peer_id = PeerId::from(peer_id);

    let mut peer_descriptor = carl.peers.get_peer_descriptor(peer_id).await
        .map_err(|_| format!("Failed to get peer with ID <{}>.", peer_id))?;

    let peer_interface_names = peer_descriptor.network_configuration.interfaces
        .iter().map(|interface| interface.name.clone()).collect::<Vec<_>>();

    let interface_name = NetworkInterfaceName::try_from(interface_name).map_err(|error| error.to_string())?;

    // TODO: Properly implement CAN parameter configuration
    let interface_configuration = match interface_type {
        NetworkInterfaceType::Ethernet => NetworkInterfaceConfiguration::Ethernet,
        NetworkInterfaceType::Can => NetworkInterfaceConfiguration::Can {
            bitrate: 500000,
            sample_point: CanSamplePoint::try_from(0.7).unwrap(),
            fd: true,
            data_bitrate: 2000000,
            data_sample_point: CanSamplePoint::try_from(0.7).unwrap(),
        },
    };

    if peer_interface_names.contains(&interface_name) {
        Err(format!("Could not create peer network configuration with name '{}' because it already exists", &interface_name))?
    } else {
        peer_descriptor.network_configuration.interfaces.push(
            NetworkInterfaceDescriptor {
                name: interface_name,
                configuration: interface_configuration,
            }
        );
    }

    carl.peers.store_peer_descriptor(Clone::clone(&peer_descriptor)).await
        .map_err(|error| format!("Failed to update peer <{}>.\n  {}", peer_id, error))?;
    let output_format = DescribeOutputFormat::from(output);
    crate::commands::peer::describe::render_peer_descriptor(peer_descriptor, output_format);

    Ok(())
}
