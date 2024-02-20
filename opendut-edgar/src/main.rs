use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::str::FromStr;

use clap::{Parser, Subcommand};
use url::Url;
use uuid::Uuid;

use opendut_edgar::setup;
use opendut_types::peer::PeerId;
use opendut_types::util::net::NetworkInterfaceName;
use opendut_types::vpn::netbird::SetupKey;

#[derive(Debug, Parser)]
#[command(name = "opendut-edgar")]
#[command(about = "Connect your ECU to openDuT.")]
#[command(long_version = opendut_edgar::app_info::formatted())]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
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
        mode: SetupMode,

        /// Run through all steps without changing the system
        #[arg(long)]
        dry_run: bool,

        /// Continue execution without asking for confirmation.
        #[arg(long)]
        no_confirm: bool,

        /// Specify the Maximum Transfer Unit for network packages in bytes.
        #[arg(long, default_value="1538")]
        mtu: u16,
    },
}

#[derive(Debug, Subcommand)]
enum SetupMode {
    /// Prepare your system for running EDGAR Service
    Managed {
        // Setup String retrieved from LEA
        #[arg()]
        setup_string: String,
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
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let args = Cli::parse();

    match args.command {
        Commands::Service { id } => {
            let id_override = id.map(PeerId::from);
            opendut_edgar::service::start::launch(
                id_override,
            ).await
        },
        Commands::Setup { mode, dry_run, no_confirm, mtu } => {
            setup::start::init_logging()?;

            let command = std::env::args_os()
                .collect::<Vec<_>>();
            log::info!("Setup command being executed: {:?}", command);

            let run_mode = if dry_run { setup::RunMode::DryRun } else { setup::RunMode::Normal };

            match mode {
                SetupMode::Managed { setup_string } => {
                    setup::start::managed(run_mode, no_confirm, setup_string, mtu).await
                },
                SetupMode::Unmanaged { management_url, setup_key, leader, bridge, device_interfaces } => {
                    let setup_key = SetupKey { uuid: setup_key };
                    let ParseableLeader(leader) = leader;
                    let bridge = bridge.unwrap_or_else(opendut_edgar::common::default_bridge_name);
                    let device_interfaces = HashSet::from_iter(device_interfaces);
                    setup::start::unmanaged(run_mode, no_confirm, management_url, setup_key, bridge, device_interfaces, leader, mtu).await
                }
            }
        },
    }
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
