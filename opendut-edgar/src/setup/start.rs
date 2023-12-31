use std::fs::File;
use std::rc::Rc;

use anyhow::Context;
use url::Url;

use opendut_types::peer::PeerSetup;
use opendut_types::topology::InterfaceName;
use opendut_types::vpn::netbird::SetupKey;
use opendut_types::vpn::VpnPeerConfig;
use opendut_util::logging;

use crate::service::network_device::manager::NetworkDeviceManager;
use crate::setup::{Router, runner, tasks};
use crate::setup::runner::RunMode;
use crate::setup::task::Task;
use crate::setup::tasks::write_configuration;

#[allow(clippy::box_default)]
pub async fn managed(run_mode: RunMode, setup_string: String, mtu: u16) -> anyhow::Result<()> {

    let peer_setup = PeerSetup::decode(&setup_string)
        .context("Failed to decode Setup String.")?;

    let (management_url, setup_key) = match peer_setup.vpn {
        VpnPeerConfig::Disabled => {
            unimplemented!("Prepare EDGAR for disabled VPN")
        }
        VpnPeerConfig::Netbird { management_url, setup_key } => {
            (management_url, setup_key)
        }
    };

    let network_device_manager = Rc::new(NetworkDeviceManager::create()?);
    let bridge_name = InterfaceName::try_from("br-opendut").unwrap();

    let tasks: Vec<Box<dyn Task>> = vec![
        Box::new(tasks::CheckOsRequirements),
        Box::new(tasks::WriteConfiguration::with_override(write_configuration::ConfigOverride {
            peer_id: peer_setup.id,
            carl_url: peer_setup.carl,
        })),
        Box::new(tasks::CheckCarlReachable),
        Box::new(tasks::CreateUser),

        //NetBird
        Box::new(tasks::netbird::Unpack::default()),
        Box::new(tasks::netbird::InstallService),
        Box::new(tasks::netbird::StartService),
        Box::new(tasks::netbird::Connect { management_url, setup_key, mtu }),

        //EDGAR Service
        Box::new(tasks::CopyExecutable),
        Box::new(tasks::ClaimFileOwnership),
        Box::new(tasks::linux_network_capability::MakePamAuthOptional::default()),
        Box::new(tasks::linux_network_capability::RequestCapabilityForUser),
        Box::new(tasks::linux_network_capability::RequestCapabilityForExecutable),
        Box::new(tasks::network_device::CreateBridge { network_device_manager: network_device_manager.clone(), bridge_name }),
        Box::new(tasks::CreateServiceFile),
        Box::new(tasks::StartService),
    ];

    runner::run(run_mode, &tasks).await
}

#[allow(clippy::box_default)]
pub async fn unmanaged(run_mode: RunMode, management_url: Url, setup_key: SetupKey, bridge_name: InterfaceName, router: Router, mtu: u16) -> anyhow::Result<()> {

    let network_device_manager = Rc::new(NetworkDeviceManager::create()?);

    let tasks: Vec<Box<dyn Task>> = vec![
        Box::new(tasks::CheckOsRequirements),
        Box::new(tasks::netbird::Unpack::default()),
        Box::new(tasks::netbird::InstallService),
        Box::new(tasks::netbird::StartService),
        Box::new(tasks::netbird::Connect { management_url, setup_key, mtu }),

        Box::new(tasks::network_device::CreateBridge { network_device_manager: network_device_manager.clone(), bridge_name: bridge_name.clone() }),
        Box::new(tasks::network_device::CreateGreInterfaces { network_device_manager: network_device_manager.clone(), bridge_name: bridge_name.clone(), router }),
        Box::new(tasks::network_device::ConnectDeviceInterfaces { network_device_manager, bridge_name }),
    ];

    runner::run(run_mode, &tasks).await
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
