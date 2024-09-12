use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Ok};
use tracing::{debug, error, info, warn};
use url::Url;

use opendut_types::peer::PeerSetup;
use opendut_types::util::net::NetworkInterfaceName;
use opendut_types::vpn::netbird::SetupKey;
use opendut_types::vpn::VpnPeerConfiguration;
use opendut_util::telemetry;
use opendut_util::telemetry::opentelemetry_types::Opentelemetry;

use crate::service::network_interface::manager::NetworkInterfaceManager;
use crate::setup::{tasks, Leader, User};
use crate::common::task::runner::RunMode;
use crate::common::task::{runner, Task};
use crate::fs;
use crate::setup::tasks::write_configuration;
use crate::setup::util::running_in_docker;

use super::constants::setup_plugins;
use super::plugin_runtime::PluginRuntime;

#[allow(clippy::box_default)]
pub async fn managed(run_mode: RunMode, no_confirm: bool, setup_string: String, mtu: u16) -> anyhow::Result<()> {

    let peer_setup = PeerSetup::decode(&setup_string)
        .context("Failed to decode Setup-String.")?;

    let service_user = determine_service_user_name();
    info!("Using service user '{}'.", service_user.name);

    println!("Using PeerId: {}", peer_setup.id);
    println!("Will connect to CARL at: {}", peer_setup.carl);

    let mut tasks: Vec<Box<dyn Task>> = vec![];

    #[cfg(target_arch = "arm")]
    {
        println!("Running on ARMv7 / ARM32. Plugins cannot be used on this architecture. https://github.com/bytecodealliance/wasmtime/issues/1173");
        info("Running on ARMv7 / ARM32. Plugins cannot be used on this architecture. https://github.com/bytecodealliance/wasmtime/issues/1173")
    }
    #[cfg(not(target_arch = "arm"))]
    let _ = create_plugin_runtime(&mut tasks);


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
                Box::new(tasks::netbird::RestartService),
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

    let mut tasks: Vec<Box<dyn Task>> = vec![];
    
    #[cfg(target_arch = "arm")]
    {
        println!("Running on ARMv7 / ARM32. Plugins cannot be used on this architecture. https://github.com/bytecodealliance/wasmtime/issues/1173");
        info("Running on ARMv7 / ARM32. Plugins cannot be used on this architecture. https://github.com/bytecodealliance/wasmtime/issues/1173")
    }
    #[cfg(not(target_arch = "arm"))]
    let _ = create_plugin_runtime(&mut tasks);

    tasks.append(&mut vec![
        Box::new(tasks::CheckCommandLinePrograms),
        Box::new(tasks::netbird::Unpack::default()),
        Box::new(tasks::netbird::InstallService),
        Box::new(tasks::netbird::RestartService),
        Box::new(tasks::netbird::Connect { management_url, setup_key, mtu }),

        Box::new(tasks::network_interface::CreateBridge { network_interface_manager: Arc::clone(&network_interface_manager), bridge_name: bridge_name.clone() }),
        Box::new(tasks::network_interface::CreateGreInterfaces { network_interface_manager: Arc::clone(&network_interface_manager), bridge_name: bridge_name.clone(), leader }),
        Box::new(tasks::network_interface::ConnectDeviceInterfaces { network_interface_manager, bridge_name, device_interfaces }),

        Box::new(tasks::copy_rperf::CopyRperf),
    ]);

    runner::run(run_mode, no_confirm, &tasks).await
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

fn create_plugin_runtime(tasks: &mut Vec<Box<dyn Task>>) -> PluginRuntime {
    use crate::setup::plugin_runtime::PluginRuntime;

    let plugin_runtime = PluginRuntime::new();
    let plugin_paths = discover_plugins().unwrap();

    let mut plugins: Vec<Box<dyn Task>> = plugin_paths.iter()
        .map(|path| Box::new(plugin_runtime.create_plugin_from_wasm(path)) as Box<dyn Task>)
        .collect();

    tasks.append(&mut plugins);

    plugin_runtime
}

fn discover_plugins() -> anyhow::Result<Vec<PathBuf>> {
	let path = setup_plugins::path_in_edgar_distribution()?;

    discover_plugins_in_path(&path)
}

fn discover_plugins_in_path(plugin_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    if !plugin_dir.exists() {
        warn!("File or folder '{}' does not exist.", plugin_dir.display());
        return Ok(vec![]);
    }

    let plugin_order = read_plugin_order(plugin_dir)?;

    let mut plugin_paths: Vec<PathBuf> = vec![];

    for entry in plugin_order {
        if entry.is_dir() {
            plugin_paths.append(&mut discover_plugins_in_path(&entry)?);
        } else if entry.extension().and_then(OsStr::to_str) == Some("wasm") {
            debug!("Found plugin at: {}", entry.display());
            plugin_paths.push(entry);
        }
    }

    Ok(plugin_paths)
}

fn read_plugin_order(plugin_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let config_path = plugin_dir.join("plugins.txt");

    if !config_path.exists() {
        warn!("No plugin configuration found at: {}", plugin_dir.display());
        return Ok(vec![]);
    }

    let config = fs::read_to_string(&config_path)?;
    let potential_plugin_paths = config.lines()
        .map(|line| line.trim())
        .filter(|line| line.is_empty())
        .map(PathBuf::from);

    let mut paths: Vec<PathBuf> = vec![];

    for mut potential_path in potential_plugin_paths {
        if !potential_path.is_absolute() {
            potential_path = plugin_dir.join(&potential_path);
        }

        if potential_path == plugin_dir {
            warn!("The plugins.txt file at '{}' refers to itself. Ignoring this reference.", config_path.display());
            continue;
        }
        
        if potential_path.exists() {
            paths.push(potential_path);
        } else {
            error!("Plugin {} specified in {}/plugins.txt does not exist", potential_path.display(), plugin_dir.display());
        }
    };

    Ok(paths)
}
