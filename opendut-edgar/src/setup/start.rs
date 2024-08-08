use std::collections::HashSet;
use std::{env, fs};
use std::ffi::OsStr;
use std::fs::FileType;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use tracing::info;
use url::Url;

use opendut_types::peer::PeerSetup;
use opendut_types::util::net::NetworkInterfaceName;
use opendut_types::vpn::netbird::SetupKey;
use opendut_types::vpn::VpnPeerConfiguration;
use opendut_util::telemetry;
use opendut_util::telemetry::opentelemetry_types::Opentelemetry;

use crate::service::network_interface::manager::NetworkInterfaceManager;
use crate::setup::plugin_runtime::PluginRuntime;
use crate::setup::{Leader, runner, tasks, User};
use crate::setup::runner::RunMode;
use crate::setup::task::Task;
use crate::setup::tasks::write_configuration;
use crate::setup::util::running_in_docker;

use super::constants::setup_plugins;

#[allow(clippy::box_default)]
pub async fn managed(run_mode: RunMode, no_confirm: bool, setup_string: String, mtu: u16) -> anyhow::Result<()> {

    let peer_setup = PeerSetup::decode(&setup_string)
        .context("Failed to decode Setup-String.")?;

    let service_user = determine_service_user_name();
    info!("Using service user '{}'.", service_user.name);

    println!("Using PeerId: {}", peer_setup.id);
    println!("Will connect to CARL at: {}", peer_setup.carl);

    let mut tasks: Vec<Box<dyn Task>> = vec![];
    
    let plugin_paths = discover_plugins().unwrap();
    
    let plugin_runtime = PluginRuntime::new();
    let mut plugins: Vec<Box<dyn Task>> = plugin_paths.iter().map(|path|Box::new(plugin_runtime.create_plugin_from_wasm(plugin_paths))).collect();

    tasks.append(&mut plugins);

    tasks.append(&mut vec![
        Box::new(tasks::WriteCaCertificate::with_certificate(peer_setup.ca)),
        Box::new(tasks::CheckCommandLinePrograms),
        Box::new(tasks::WriteConfiguration::with_override(
            write_configuration::ConfigOverride {
                peer_id: peer_setup.id,
                carl_url: peer_setup.carl,
                auth_config: peer_setup.auth_config,
            }),
        ),
        Box::new(tasks::CheckCarlReachable),
        Box::new(tasks::CopyExecutable),
        Box::new(tasks::copy_rperf::CopyRperf),

        Box::new(tasks::LoadKernelModules::default()),
    ]);

    if !running_in_docker() {
        tasks.push(Box::new(tasks::CreateKernelModuleLoadRule))
    }

    match peer_setup.vpn {
        VpnPeerConfiguration::Disabled => {
            info!("VPN is disabled in PeerSetup. Not running VPN-related tasks.");
        }
        VpnPeerConfiguration::Netbird { management_url, setup_key } => {
            info!("VPN is configured for NetBird in PeerSetup. Running NetBird-related tasks.");
            tasks.append(&mut vec![
                Box::new(tasks::netbird::Unpack::default()),
                Box::new(tasks::netbird::InstallService),
                Box::new(tasks::netbird::StartService),
                Box::new(tasks::netbird::Connect { management_url, setup_key, mtu }),
            ]);
        }
    };

    if service_user.is_root() {
        info!("Service should run via root user. Skipping setup of custom service user.");
    } else {
        info!("Setting up custom service user '{}'.", service_user.name);
        tasks.append(&mut vec![
            Box::new(tasks::CreateUser { service_user: service_user.clone() }),
            Box::new(tasks::ClaimFileOwnership { service_user: service_user.clone() }),
            Box::new(tasks::RequestLinuxNetworkCapability),
        ]);
    }

    tasks.append(&mut vec![
        Box::new(tasks::CreateServiceFile::with_service_user(service_user)),
        Box::new(tasks::RestartService),
    ]);

    runner::run(run_mode, no_confirm, &tasks, plugin_runtime).await
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

    let mut tasks: Vec<Box<dyn Task>> = vec![];
    
    let plugin_paths = discover_plugins().unwrap();
    
    let plugin_runtime = PluginRuntime::new();
    let mut plugins: Vec<Box<dyn Task>> = plugin_paths.iter().map(|path|Box::new(plugin_runtime.create_plugin_from_wasm(plugin_paths))).collect();

    tasks.append(&mut plugins);

    tasks.append(&mut vec![
        Box::new(tasks::CheckCommandLinePrograms),
        Box::new(tasks::netbird::Unpack::default()),
        Box::new(tasks::netbird::InstallService),
        Box::new(tasks::netbird::StartService),
        Box::new(tasks::netbird::Connect { management_url, setup_key, mtu }),
        
        Box::new(tasks::network_interface::CreateBridge { network_interface_manager: Arc::clone(&network_interface_manager), bridge_name: bridge_name.clone() }),
        Box::new(tasks::network_interface::CreateGreInterfaces { network_interface_manager: Arc::clone(&network_interface_manager), bridge_name: bridge_name.clone(), leader }),
        Box::new(tasks::network_interface::ConnectDeviceInterfaces { network_interface_manager, bridge_name, device_interfaces }),

        Box::new(tasks::copy_rperf::CopyRperf),
    ]);

    runner::run(run_mode, no_confirm, &tasks, plugin_runtime).await
}


pub async fn init_logging() -> anyhow::Result<()> {

    let mut log_file = std::env::current_exe().unwrap();
    log_file.set_file_name("setup.log");
    let file_logging = Some(log_file);

    let logging_config = telemetry::logging::LoggingConfig {
        logging_stdout: false,
        file_logging,
    };
    let opentelemetry_config = Opentelemetry::Disabled;
    
    let _ = telemetry::initialize_with_config(logging_config, opentelemetry_config).await?;

    Ok(())
}

fn determine_service_user_name() -> User {
    const DEFAULT_SERVICE_USER_NAME: &str = "opendut";

    let name = env::var("OPENDUT_EDGAR_SERVICE_USER")
        .unwrap_or(DEFAULT_SERVICE_USER_NAME.to_string());

    User { name }
}

fn discover_plugins() -> anyhow::Result<Vec<PathBuf>> {
	let path = setup_plugins::path_in_edgar_distribution()?;
	let paths = fs::read_dir(&path)?;

	let result: Vec<PathBuf> = paths
		.filter_map(|item| {
			item.ok().and_then(|entry| {
				if entry.path().extension().and_then(OsStr::to_str) == Some("wasm") {
					Some(entry.path())
				} else {
					None
				}
			})
		})
		.collect();

	Ok(result)
}
