pub mod logging;

use tracing::info;

use crate::common::task::runner::RunMode;
use crate::common::task::{runner, Task};
use crate::setup::write_configuration;
use crate::setup::util::running_in_docker;
use crate::setup::{tasks, User};
use opendut_model::peer::PeerSetup;
use opendut_model::vpn::VpnPeerConfiguration;
use std::env;
use std::ops::Not;
use crate::interactive_message;
use crate::setup::cli::SetupRunCommonArgs;
use crate::setup::util::DryRun;


#[allow(clippy::box_default)]
pub(super) async fn managed(
    peer_setup: PeerSetup,
    common_args: SetupRunCommonArgs,
) -> anyhow::Result<()> {
    let SetupRunCommonArgs { dry_run, no_confirm, log_file: _, mtu, skip_can, skip_service_run } = common_args;

    let service_user = determine_service_user_name();
    info!("Using service user '{}'.", service_user.name);

    interactive_message!("Using PeerId: {}", peer_setup.id);
    interactive_message!("Will connect to CARL at: {}", peer_setup.carl);

    let should_run = no_confirm || user_confirmation(&dry_run)?;
    if should_run.not() {
        return Ok(());
    }

    if dry_run.not() {
        write_configuration::write_with_override(
            write_configuration::ConfigOverride {
                peer_id: peer_setup.id,
                carl_url: peer_setup.carl,
                auth_config: peer_setup.auth_config,
            },
            no_confirm,
        )?;
    }

    let mut tasks: Vec<Box<dyn Task>> = vec![];

    let _ = crate::setup::plugin::init::create_plugin_runtime(&mut tasks)?;


    tasks.append(&mut vec![
        Box::new(tasks::WriteCaCertificate::with_certificate(peer_setup.ca)),
        Box::new(tasks::CheckCommandLinePrograms { skip_can, skip_service_run }),
        Box::new(tasks::CheckCarlReachable),
        Box::new(tasks::CopyExecutable),
        Box::new(tasks::copy_rperf::CopyRperf),
    ]);

    if skip_can.not() {
        tasks.push(Box::new(tasks::LoadCanKernelModules::default()));

        if !running_in_docker() {
            tasks.push(Box::new(tasks::CreateCanKernelModuleLoadRule))
        }
    }

    match peer_setup.vpn {
        VpnPeerConfiguration::Disabled => {
            info!("VPN is disabled in PeerSetup. Not running VPN-related tasks.");
        }
        VpnPeerConfiguration::Netbird { management_url, setup_key } => {
            info!("VPN is configured for NetBird in PeerSetup. Running NetBird-related tasks.");
            tasks.append(&mut vec![
                Box::new(tasks::netbird::Unpack::default()),
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

    if skip_service_run.not() {
        tasks.append(&mut vec![
            Box::new(tasks::CreateServiceFile::with_service_user(service_user)),
            Box::new(tasks::RestartService),
        ]);
    }

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

fn determine_service_user_name() -> User {
    const DEFAULT_SERVICE_USER_NAME: &str = "opendut_service";

    let name = env::var("OPENDUT_EDGAR_SERVICE_USER")
        .unwrap_or(DEFAULT_SERVICE_USER_NAME.to_string());

    User { name }
}

fn user_confirmation(dry_run: &DryRun) -> anyhow::Result<bool> {
    let crate_version = crate::app_info::PKG_VERSION;
    match dry_run {
        DryRun::No => {
            interactive_message!("This will setup EDGAR {crate_version} on your system.");

            let user_confirmed = crate::setup::user_confirmation_prompt("Do you want to continue?")?;

            if user_confirmed.not() {
                interactive_message!("Aborting.");
                info!("Aborting, because user did not confirm execution.");
            }
            Ok(user_confirmed)
        }
        DryRun::Yes => {
            interactive_message!("Pretending to setup EDGAR {crate_version} on your system.");
            Ok(true)
        }
    }
}
