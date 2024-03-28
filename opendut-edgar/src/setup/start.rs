use std::collections::HashSet;
use std::env;
use std::sync::Arc;

use anyhow::Context;
use url::Url;

use opendut_types::peer::PeerSetup;
use opendut_types::util::net::NetworkInterfaceName;
use opendut_types::vpn::netbird::SetupKey;
use opendut_types::vpn::VpnPeerConfiguration;
use opendut_util::logging;

use crate::service::network_interface::manager::NetworkInterfaceManager;
use crate::setup::{Leader, runner, tasks, User};
use crate::setup::runner::RunMode;
use crate::setup::task::Task;
use crate::setup::tasks::write_configuration;
use crate::setup::util::running_in_docker;

#[allow(clippy::box_default)]
pub async fn managed(run_mode: RunMode, no_confirm: bool, setup_string: String, mtu: u16) -> anyhow::Result<()> {

    let peer_setup = PeerSetup::decode(&setup_string)
        .context("Failed to decode Setup String.")?;

    let service_user = determine_service_user_name();
    log::info!("Using service user '{}'.", service_user.name);

    println!("Using PeerId: {}", peer_setup.id);
    println!("Will connect to CARL at: {}", peer_setup.carl);

    let mut tasks: Vec<Box<dyn Task>> = vec![
        Box::new(tasks::WriteCaCertificate { certificate: peer_setup.ca }),
        Box::new(tasks::CheckOsRequirements),
        Box::new(tasks::WriteConfiguration::with_override(write_configuration::ConfigOverride {
            peer_id: peer_setup.id,
            carl_url: peer_setup.carl,
        })),
        Box::new(tasks::CheckCarlReachable),
        Box::new(tasks::CopyExecutable),
        Box::new(tasks::LoadKernelModules::default()),
    ];

    if !running_in_docker() {
        tasks.push(Box::new(tasks::CreateKernelModuleLoadRule))
    }

    match peer_setup.vpn {
        VpnPeerConfiguration::Disabled => {
            log::info!("VPN is disabled in PeerSetup. Not running VPN-related tasks.");
        }
        VpnPeerConfiguration::Netbird { management_url, setup_key } => {
            log::info!("VPN is configured for NetBird in PeerSetup. Running NetBird-related tasks.");
            tasks.append(&mut vec![
                Box::new(tasks::netbird::Unpack::default()),
                Box::new(tasks::netbird::InstallService),
                Box::new(tasks::netbird::StartService),
                Box::new(tasks::netbird::Connect { management_url, setup_key, mtu }),
            ]);
        }
    };

    if service_user.is_root() {
        log::info!("Service should run via root user. Skipping setup of custom service user.");
    } else {
        log::info!("Setting up custom service user '{}'.", service_user.name);
        tasks.append(&mut vec![
            Box::new(tasks::CreateUser { service_user: service_user.clone() }),
            Box::new(tasks::ClaimFileOwnership { service_user: service_user.clone() }),
            Box::new(tasks::RequestLinuxNetworkCapability),
        ]);
    }

    tasks.append(&mut vec![
        Box::new(tasks::CreateServiceFile { service_user }),
        Box::new(tasks::StartService),
    ]);

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

    let network_interface_manager = NetworkInterfaceManager::create()?;

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

    let log_file = std::env::current_exe().unwrap()
        .parent().unwrap()
        .join("setup.log");

    let config = logging::LoggingConfig {
        file_logging: Some(log_file),
        opentelemetry_endpoint: None,
        opentelemetry_service_name: None,
        opentelemetry_service_instance_id: None,
    };
    let _ = logging::initialize_with_config(config)?;

    Ok(())
}

fn determine_service_user_name() -> User {
    const DEFAULT_SERVICE_USER_NAME: &str = "opendut";

    let name = env::var("OPENDUT_EDGAR_SERVICE_USER")
        .unwrap_or(DEFAULT_SERVICE_USER_NAME.to_string());

    User { name }
}
