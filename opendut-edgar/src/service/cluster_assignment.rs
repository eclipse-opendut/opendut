use tracing::debug;

use crate::service::can_manager::CanManagerRef;
use opendut_types::cluster::{ClusterAssignment, PeerClusterAssignment};
use opendut_types::peer::configuration::{parameter, Parameter, ParameterTarget};
use opendut_types::peer::PeerId;
use opendut_types::util::net::NetworkInterfaceConfiguration;
use parameter::DeviceInterface;

#[tracing::instrument(skip_all, level="trace")]
pub async fn setup_can_interfaces(
    cluster_assignment: &ClusterAssignment,
    self_id: PeerId,
    device_interfaces: &[Parameter<DeviceInterface>],
    can_manager: CanManagerRef
) -> Result<(), Error> {
    let can_interfaces = filter_can_interfaces(device_interfaces.to_owned())?;

    let can_interfaces = can_interfaces.into_iter()
        .filter(|parameter| parameter.target == ParameterTarget::Present)
        .map(|parameter| parameter.value.descriptor)
        .collect::<Vec<_>>();

    if let sudo::RunningAs::User = sudo::check() {
        if can_interfaces.is_empty() {
            //Since we don't have the correct permissions to run the CAN setup code,
            //no previous CAN interfaces exist which we might need to clean up,
            //so we can safely skip this code, which allows us to run without root,
            //when CAN is not used.
            debug!("No CAN interfaces to set up. Skipping.");
            return Ok(());
        } else {
            panic!("CARL requested to setup CAN interfaces, but EDGAR is not running with root permissions, which is currently required."); //TODO report problem to CARL
        }
    }

    debug!("Setting up CAN interfaces.");

    let can_bridge_name = crate::common::default_can_bridge_name();
    can_manager.setup_local_routing(
        &can_bridge_name,
        can_interfaces,
    ).await
    .map_err(Error::LocalCanRoutingSetupFailed)?;

    let local_peer_assignment = cluster_assignment.assignments.iter().find(|assignment| {
        assignment.peer_id == self_id
    }).ok_or(Error::LocalPeerAssignmentNotFound { self_id })?;

    let is_leader = cluster_assignment.leader == self_id;

    let server_port = local_peer_assignment.can_server_port;

    if is_leader {

        let remote_assignments = determine_remote_assignments(cluster_assignment, self_id)?;
        can_manager.setup_remote_routing_server(
            &can_bridge_name, 
            &remote_assignments
        ).await
        .map_err(Error::RemoteCanRoutingSetupFailed)?;

    } else {        

        let leader_assignment = determine_leader_assignment(cluster_assignment)?;
        can_manager.setup_remote_routing_client(
            &can_bridge_name, 
            &leader_assignment.vpn_address,
            &server_port
        ).await
        .map_err(Error::RemoteCanRoutingSetupFailed)?;
    }

    Ok(())
}

fn determine_remote_assignments(cluster_assignment: &ClusterAssignment, self_id: PeerId) -> Result<Vec<PeerClusterAssignment>, Error> {
    let is_leader = cluster_assignment.leader == self_id;

    let remote_peer_cluster_assignments = if is_leader {
        cluster_assignment.assignments.iter()
            .filter(|assignment| assignment.peer_id != self_id).cloned()
            .collect::<Vec<PeerClusterAssignment>>()
    }
    else {
        let leader_ip = determine_leader_assignment(cluster_assignment)?;

        vec![leader_ip.clone()]
    };

    Ok(remote_peer_cluster_assignments)
}

fn determine_leader_assignment(cluster_assignment: &ClusterAssignment) -> Result<&PeerClusterAssignment, Error>{
    let leader_assignment = cluster_assignment.assignments
        .iter().find(|peer_assignment| 
            peer_assignment.peer_id == cluster_assignment.leader
            ).ok_or(Error::LeaderNotDeterminable)?;

    Ok(leader_assignment)
}

fn filter_can_interfaces(
    device_interfaces: Vec<Parameter<DeviceInterface>>
) -> Result<Vec<Parameter<DeviceInterface>>, Error> {

    let own_can_interfaces: Vec<_> = device_interfaces.iter()
        .filter(|interface| matches!(interface.value.descriptor.configuration, NetworkInterfaceConfiguration::Can { .. }))
        .cloned()
        .collect::<Vec<_>>();

    Ok(own_can_interfaces)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not find PeerAssignment for this peer (<{self_id}>) in the ClusterAssignment.")]
    LocalPeerAssignmentNotFound { self_id: PeerId },
    #[error("Could not determine leader from ClusterAssignment.")]
    LeaderNotDeterminable,
    #[error("Local CAN routing setup failed: {0}")]
    LocalCanRoutingSetupFailed(crate::service::can_manager::Error),
    #[error("Remote CAN routing setup failed: {0}")]
    RemoteCanRoutingSetupFailed(crate::service::can_manager::Error),
}
