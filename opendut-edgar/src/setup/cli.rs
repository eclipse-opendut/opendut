use crate::{interactive_message, setup};
use anyhow::{bail, Context};
use clap::{Args, Subcommand};
use opendut_model::peer::PeerSetup;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use tracing::{debug, info};
use crate::setup::start::logging;
use crate::setup::util::DryRun;

const SETUP_STRING_ENV: &str = "OPENDUT_EDGAR_SETUP_STRING";


/// Prepare your system
#[derive(clap::Parser)]
pub struct SetupCli {
    #[command(subcommand)]
    command: SetupCommand,
}

#[derive(Subcommand)]
enum SetupCommand {
    /// Prepare your system for running EDGAR Service
    Managed {
        // Setup String retrieved from LEA
        #[arg()]
        setup_string: Option<String>,

        #[clap(flatten)]
        common: SetupRunCommonArgs,
    },
}

#[derive(Args)]
pub(super) struct SetupRunCommonArgs {
    /// Run through all steps without changing the system
    #[arg(long, global=true, default_value="false")]
    pub dry_run: DryRun,

    /// Continue execution without asking for confirmation.
    #[arg(long, global=true)]
    pub no_confirm: bool,

    /// The path to store logs at. Pass `-` to write to stderr instead.
    /// If nothing is specified, the logs will be written to a `setup.log` file, relative to the executable.
    #[arg(long, global=true)]
    pub log_file: Option<PathBuf>,

    /// Specify the Maximum Transfer Unit for network packages in bytes.
    #[arg(long, global=true, default_value="1542")]
    pub mtu: u16,

    /// EXPERT OPTION: Do not setup CAN on this host and skip checks for it.
    /// Users need to know not to use CAN on this EDGAR, otherwise this can lead to undefined behavior and crashes.
    #[arg(long, global=true, alias="skip_can_setup")] //was called "skip_can_setup" before 2026-01-26; keeping it for backwards compatibility for now
    pub skip_can: bool,

    /// EXPERT OPTION: Do not run the EDGAR Service at the end of the setup and skip checks for it.
    /// Primarily intended for use when running EDGAR in a container.
    #[arg(long, global=true, hide=true)]
    pub skip_service_run: bool,
}

impl SetupCli {
    pub async fn run(self) -> anyhow::Result<ExitCode> {
        let exit_code =
            match self.command {
                SetupCommand::Managed { setup_string, common } => {
                    logging::init(&common.log_file).await?;

                    let user_command = env::args_os()
                        .collect::<Vec<_>>();
                    info!("EDGAR Setup started!");
                    info!("Setup command being executed: {:?}", user_command);

                    let peer_setup = parse_peer_setup(setup_string)?;

                    setup::start::managed(peer_setup, common).await?
                },
            };
        Ok(exit_code)
    }
}


fn parse_peer_setup(setup_string_via_arg: Option<String>) -> anyhow::Result<PeerSetup> {

    let setup_string =
        if let Some(setup_string_via_arg) = setup_string_via_arg {
            setup_string_via_arg
        }
        else if let Some(setup_string_via_env) = env::var_os(SETUP_STRING_ENV) {
            match setup_string_via_env.into_string() {
                Ok(setup_string_via_env) => setup_string_via_env,
                Err(_os_string) => bail!("Error while reading from {SETUP_STRING_ENV} as UTF-8."),
            }
        }
        else {
            if console::user_attended() && logging::interactive_messages::is_enabled() {
                interactive_message!("You can retrieve a Setup-String from the web-UI.");
                interactive_message!("Enter your Setup-String here:");
            } else {
                debug!("Reading Setup-String from stdin.");
            }

            let mut setup_string = String::new();

            std::io::stdin().read_line(&mut setup_string)
                .context("Error while reading Setup-String from stdin.")?;

            setup_string.trim().to_owned()
        };

    if setup_string.is_empty() {
        bail!("No Setup-String provided. You can pass it via environment variable {SETUP_STRING_ENV}, command-line argument or stdin.");
    } else {
        let peer_setup = PeerSetup::decode(&setup_string)
            .context("Failed to decode Setup-String.")?;

        Ok(peer_setup)
    }
}
