use std::collections::HashSet;
use std::fs::File;
use std::sync::Arc;

use anyhow::Context;
use url::Url;

use opendut_types::peer::PeerSetup;
use opendut_types::util::net::NetworkInterfaceName;
use opendut_types::vpn::netbird::SetupKey;
use opendut_types::vpn::VpnPeerConfiguration;
use opendut_util::logging;

use crate::service::network_interface::manager::NetworkInterfaceManager;
use crate::setup::{Leader, runner, tasks};
use crate::setup::runner::RunMode;
use crate::setup::task::Task;
use crate::setup::tasks::write_configuration;

#[allow(clippy::box_default)]
pub async fn managed(run_mode: RunMode, no_confirm: bool, setup_string: String, mtu: u16) -> anyhow::Result<()> {

    let peer_setup = PeerSetup::decode(&setup_string)
        .context("Failed to decode Setup String.")?;

    println!("Using PeerId: {}", peer_setup.id);
    println!("Will connect to CARL at: {}", peer_setup.carl);

    let mut tasks: Vec<Box<dyn Task>> = vec![
        Box::new(tasks::CheckOsRequirements),
        Box::new(tasks::WriteConfiguration::with_override(write_configuration::ConfigOverride {
            peer_id: peer_setup.id,
            carl_url: peer_setup.carl,
        })),
        Box::new(tasks::CheckCarlReachable),
        Box::new(tasks::CreateUser),
    ];

    match peer_setup.vpn {
        VpnPeerConfiguration::Disabled => {
            log::info!("VPN is disabled in PeerSetup. Not running VPN-related tasks.");
        }
        VpnPeerConfiguration::Netbird { management_url, setup_key } => {
            log::info!("VPN is configured for NetBird in PeerSetup. Running NetBird-related tasks.");
            let mut netbird_tasks: Vec<Box<dyn Task>> = vec![
                Box::new(tasks::netbird::Unpack::default()),
                Box::new(tasks::netbird::InstallService),
                Box::new(tasks::netbird::StartService),
                Box::new(tasks::netbird::Connect { management_url, setup_key, mtu }),
            ];
            tasks.append(&mut netbird_tasks);
        }
    };

    let mut service_tasks: Vec<Box<dyn Task>> = vec![
        Box::new(tasks::CopyExecutable),
        Box::new(tasks::ClaimFileOwnership),
        Box::new(tasks::linux_network_capability::MakePamAuthOptional::default()),
        Box::new(tasks::linux_network_capability::RequestCapabilityForUser),
        Box::new(tasks::linux_network_capability::RequestCapabilityForExecutable),
        Box::new(tasks::CreateServiceFile),
        Box::new(tasks::StartService),
    ];
    tasks.append(&mut service_tasks);

    runner::run(run_mode, no_confirm, &tasks).await
}

#[allow(clippy::box_default, clippy::too_many_arguments)]
pub async fn unmanaged(
    run_mode: RunMode,
    no_confirm: bool,
    management_url: Url,
    setup_key: SetupKey,
    bridge_name: NetworkInterfaceName,
    device_interfaces: HashSet<NetworkInterfaceName>,
    leader: Leader,
    mtu: u16,
) -> anyhow::Result<()> {

    let network_interface_manager = Arc::new(NetworkInterfaceManager::create()?);

    let tasks: Vec<Box<dyn Task>> = vec![
        Box::new(tasks::CheckOsRequirements),
        Box::new(tasks::netbird::Unpack::default()),
        Box::new(tasks::netbird::InstallService),
        Box::new(tasks::netbird::StartService),
        Box::new(tasks::netbird::Connect { management_url, setup_key, mtu }),

        Box::new(tasks::network_interface::CreateBridge { network_interface_manager: Arc::clone(&network_interface_manager), bridge_name: bridge_name.clone() }),
        Box::new(tasks::network_interface::CreateGreInterfaces { network_interface_manager: Arc::clone(&network_interface_manager), bridge_name: bridge_name.clone(), leader }),
        Box::new(tasks::network_interface::ConnectDeviceInterfaces { network_interface_manager, bridge_name, device_interfaces }),
    ];

    runner::run(run_mode, no_confirm, &tasks).await
}


pub fn init_logging() -> anyhow::Result<()> {
    logging::initialize_with_overrides(|builder| {
        let log_file = std::env::current_exe().unwrap()
            .parent().unwrap()
            .join("setup.log");

        let log_file = File::create(&log_file)
            .unwrap_or_else(|cause| panic!("Failed to open log file at '{}': {cause}", log_file.display()));

        builder
            .target(env_logger::Target::Pipe(
                Box::new(log_file)
            )
        )
    })?;
    Ok(())
}
