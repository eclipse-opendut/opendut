use std::collections::HashSet;
use std::fs;
use std::net::Ipv4Addr;
use std::ops::Not;
use std::str::FromStr;

use clap::{Args, Parser, Subcommand};
use tracing::info;
use url::Url;
use uuid::Uuid;

pub use dry_run::DryRun;
use crate::setup;
use opendut_model::peer::PeerId;
use opendut_model::util::net::NetworkInterfaceName;
use opendut_model::vpn::netbird::SetupKey;


#[derive(Parser)]
#[command(name = "opendut-edgar")]
#[command(about = "Connect your ECU to openDuT.")]
#[command(long_version = crate::FORMATTED_VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launches the EDGAR Service
    Service {
        /// Start with the provided ID
        #[arg(long)]
        id: Option<Uuid>,
    },
    /// Prepare your system
    Setup {
        #[command(subcommand)]
        command: SetupCommand,
    },
}

#[derive(Subcommand)]
enum SetupCommand {
    /// Prepare your system for running EDGAR Service
    Managed {
        // Setup String retrieved from LEA
        #[arg()]
        setup_string: String,

        #[clap(flatten)]
        common: SetupRunCommonArgs,
    },
    /// Setup your system for network routing without automatic management. This setup method will be removed in the future.
    Unmanaged {
        /// URL of the VPN management service
        #[arg(long)]
        management_url: Url,

        /// Setup Key retrieved from the VPN management UI
        #[arg(long)]
        setup_key: Uuid,

        /// Whether this EDGAR should act as the leader of this network or use another EDGAR for routing (specify "local" or the IP address of the routing EDGAR respectively)
        #[arg(long, value_name="local|IP_ADDRESS")]
        leader: ParseableLeader, // We create a star topology to avoid loops between the GRE interfaces.

        /// Names of the device interfaces where the ECUs are connected
        #[arg(long, required=true)]
        device_interfaces: Vec<NetworkInterfaceName>,

        /// Name of the bridge to use, maximum 15 characters long
        #[arg(long)]
        bridge: Option<NetworkInterfaceName>,

        #[clap(flatten)]
        common: SetupRunCommonArgs,
    },
    /// Prints the logs from previous setup runs.
    Logs,
}

#[derive(Args)]
struct SetupRunCommonArgs {
    /// Run through all steps without changing the system
    #[arg(long, global=true)]
    dry_run: DryRun,

    /// Continue execution without asking for confirmation.
    #[arg(long, global=true)]
    no_confirm: bool,

    /// Specify the Maximum Transfer Unit for network packages in bytes.
    #[arg(long, global=true, default_value="1542")]
    mtu: u16,
}

pub async fn cli() -> anyhow::Result<()> {

    let args = Cli::parse();

    match args.command {
        Commands::Service { id } => {
            let id_override = id.map(PeerId::from);
            crate::service::start::launch(
                id_override,
            ).await
        },
        Commands::Setup { command } => {
            match command {
                SetupCommand::Managed { setup_string, common } => {
                    setup_run_common_prelude().await?;

                    let SetupRunCommonArgs { dry_run, no_confirm, mtu } = common;
                    setup::start::managed(dry_run, no_confirm, setup_string, mtu).await?;
                },
                SetupCommand::Unmanaged { management_url, setup_key, leader, bridge, device_interfaces, common } => {
                    setup_run_common_prelude().await?;

                    let setup_key = SetupKey { uuid: setup_key };
                    let ParseableLeader(leader) = leader;
                    let bridge = bridge.unwrap_or_else(crate::common::default_bridge_name);
                    let device_interfaces = HashSet::from_iter(device_interfaces);
                    let SetupRunCommonArgs { dry_run, no_confirm, mtu } = common;
                    setup::start::unmanaged(dry_run, no_confirm, management_url, setup_key, bridge, device_interfaces, leader, mtu).await?;
                }
                SetupCommand::Logs => {
                    let logs = fs::read_to_string(setup::start::logging_file()?)?;

                    if logs.is_empty().not() {
                        print!("{logs}");
                    } else {
                        println!("No logs found."); //TODO test
                    }
                }
            };
            info!("EDGAR Setup finished!\n");
            Ok(())
        }
    }
}

async fn setup_run_common_prelude() -> anyhow::Result<()> {
    setup::start::init_logging().await?;

    let user_command = std::env::args_os()
        .collect::<Vec<_>>();
    info!("EDGAR Setup started!");
    info!("Setup command being executed: {:?}", user_command);

    #[cfg(target_arch = "arm")]
    {
        println!("Running on ARMv7 / ARM32. Plugins cannot be used on this architecture.");
        info!("Running on ARMv7 / ARM32. Plugins cannot be used on this architecture. For more information, see: https://github.com/bytecodealliance/wasmtime/issues/1173")
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct ParseableLeader(setup::Leader);
impl FromStr for ParseableLeader {
    type Err = String;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let local_string = "local";

        if string.to_lowercase() == local_string {
            Ok(ParseableLeader(setup::Leader::Local))
        } else {
            let ip = Ipv4Addr::from_str(string)
                .map_err(|cause| format!("Specify either '{local_string}' or a valid IPv4 address ({cause})."))?;
            Ok(ParseableLeader(setup::Leader::Remote(ip)))
        }
    }
}


mod dry_run {
    use opendut_util::project;
    use tracing::info;

    #[derive(Clone, PartialEq, Eq)]
    pub enum DryRun { Yes, No }
    impl DryRun {
        pub fn not(&self) -> bool {
            self == &DryRun::No
        }

        fn force_dry_run_in_development(&mut self) {
            if project::is_running_in_development() {
                println!("{DEVELOPMENT_DRY_RUN_BANNER}");
                info!("{DEVELOPMENT_DRY_RUN_BANNER}");
                *self = DryRun::Yes;
            }
        }
    }
    impl std::str::FromStr for DryRun {
        type Err = anyhow::Error;

        fn from_str(value: &str) -> Result<Self, Self::Err> {
            let dry_run = bool::from_str(value)?;

            let mut dry_run = if dry_run { DryRun::Yes } else { DryRun::No };
            dry_run.force_dry_run_in_development();

            if dry_run.not() {
                sudo::with_env(&["OPENDUT_EDGAR_"]) //Request before doing anything else, as it restarts the process when sudo is not present.
                    .expect("Failed to request sudo privileges.");
            }
            Ok(dry_run)
        }
    }

    const DEVELOPMENT_DRY_RUN_BANNER: &str = r"
                Running in
             Development mode
                   ----
          Activating --dry-run to
        prevent changes to the system.
        ";
}
