use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Context;
use tracing::info;
use url::Url;

use crate::common::task::runner::RunMode;
use crate::common::task::{runner, Task};
use crate::service::network_interface::manager::NetworkInterfaceManager;
use crate::setup::write_configuration;
use crate::setup::util::running_in_docker;
use crate::setup::{tasks, Leader, User};
use opendut_types::peer::PeerSetup;
use opendut_types::util::net::NetworkInterfaceName;
use opendut_types::vpn::netbird::SetupKey;
use opendut_types::vpn::VpnPeerConfiguration;
use opendut_util::telemetry;
use opendut_util::telemetry::opentelemetry_types::Opentelemetry;
use std::env;
use std::ops::Not;
use opendut_util::telemetry::logging::PipeLogging;
use crate::cli::DryRun;

#[allow(clippy::box_default)]
pub async fn managed(dry_run: DryRun, no_confirm: bool, setup_string: String, mtu: u16) -> anyhow::Result<()> {

    let peer_setup = PeerSetup::decode(&setup_string)
        .context("Failed to decode Setup-String.")?;

    let service_user = determine_service_user_name();
    info!("Using service user '{}'.", service_user.name);

    println!("Using PeerId: {}", peer_setup.id);
    println!("Will connect to CARL at: {}", peer_setup.carl);

    let should_run = no_confirm || user_confirmation(&dry_run)?;
    if should_run.not() {
        return Ok(());
    }

    if dry_run.not() {
        write_configuration::WriteConfiguration::with_override(
            write_configuration::ConfigOverride {
                peer_id: peer_setup.id,
                carl_url: peer_setup.carl,
                auth_config: peer_setup.auth_config,
            },
            no_confirm,
        ).execute().await?;
    }

    let mut tasks: Vec<Box<dyn Task>> = vec![];

    #[cfg(not(target_arch = "arm"))]
    let _ = crate::setup::plugin::init::create_plugin_runtime(&mut tasks)?;


    tasks.append(&mut vec![
        Box::new(tasks::WriteCaCertificate::with_certificate(peer_setup.ca)),
        Box::new(tasks::CheckCommandLinePrograms),
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

    let run_mode = match dry_run {
        DryRun::Yes => RunMode::SetupDryRun,
        DryRun::No => RunMode::Setup,
    };
    let result = runner::run(run_mode, &tasks).await;
    if let Err(error) = result {
        error.print_error();
        // TODO: exit code
    }
    Ok(())
}

#[allow(clippy::box_default, clippy::too_many_arguments)]
pub async fn unmanaged(
    dry_run: DryRun,
    no_confirm: bool,
    management_url: Url,
    setup_key: SetupKey,
    bridge_name: NetworkInterfaceName,
    device_interfaces: HashSet<NetworkInterfaceName>,
    leader: Leader,
    mtu: u16,
) -> anyhow::Result<()> {
    let should_run = no_confirm || user_confirmation(&dry_run)?;
    if should_run.not() {
        return Ok(());
    }

    let network_interface_manager = NetworkInterfaceManager::create()?;

    let mut tasks: Vec<Box<dyn Task>> = vec![];
    
    #[cfg(not(target_arch = "arm"))]
    let _ = crate::setup::plugin::init::create_plugin_runtime(&mut tasks)?;

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

    let run_mode = match dry_run {
        DryRun::Yes => RunMode::SetupDryRun,
        DryRun::No => RunMode::Setup,
    };
    let result = runner::run(run_mode, &tasks).await;
    if let Err(error) = result {
        error.print_error();
        // TODO: exit code
    }
    Ok(())
}


pub async fn init_logging() -> anyhow::Result<()> {

    let mut log_file = env::current_exe()?;
    log_file.set_file_name("setup.log");
    let file_logging = Some(log_file);

    let logging_config = telemetry::logging::LoggingConfig {
        pipe_logging: PipeLogging::Disabled,
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

fn user_confirmation(dry_run: &DryRun) -> anyhow::Result<bool> {
    let crate_version = crate::app_info::PKG_VERSION;
    match dry_run {
        DryRun::No => {
            println!("This will setup EDGAR {crate_version} on your system.");

            let user_confirmed = crate::setup::user_confirmation_prompt("Do you want to continue?")?;

            if user_confirmed.not() {
                println!("Aborting.");
                info!("Aborting, because user did not confirm execution.");
            }
            Ok(user_confirmed)
        }
        DryRun::Yes => {
            println!("Pretending to setup EDGAR {crate_version} on your system.");
            Ok(true)
        }
    }
}
