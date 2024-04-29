use std::ops::Not;
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;

use clap::{Parser, Subcommand, ValueEnum};
use console::Style;

use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::PeerSetup;
use opendut_types::topology::DeviceName;
use opendut_util::settings::{FileFormat, load_config};

mod commands;
pub mod parse;

type Error = String;
type Result<T> = std::result::Result<T, Error>;

opendut_util::app_info!();

/// CLEO is a command line tool to manage openDuT resources.
#[derive(Parser)]
#[command(author, about, long_about = None)]
#[command(long_version = crate::app_info::formatted())]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    ///Display openDuT resources
    List {
        #[command(subcommand)]
        resource: ListResource,
        ///JSON, prettified JSON or table as output format
        #[arg(value_enum, short, long, default_value_t=ListOutputFormat::Table)]
        output: ListOutputFormat,
    },
    ///Create openDuT resource
    Create {
        #[command(subcommand)]
        resource: CreateResource,
        ///Text, JSON or prettified JSON as output format
        #[arg(value_enum, short, long, default_value_t=CreateOutputFormat::Text)]
        output: CreateOutputFormat,
    },
    GeneratePeerSetup(commands::generate_peer_setup::GeneratePeerSetupCli),
    DecodePeerSetup(commands::decode_peer_setup::DecodePeerSetupCli),
    ///Describe openDuT resource
    Describe {
        ///Name of openDuT resource
        #[command(subcommand)]
        resource: DescribeResource,
        ///JSON, prettified JSON or table as output format
        #[arg(value_enum, short, long, default_value_t=DescribeOutputFormat::Text)]
        output: DescribeOutputFormat,
    },
    ///Find openDuT resource
    Find {
        ///Name of openDuT resource
        #[command(subcommand)]
        resource: FindResource,
        ///JSON, prettified JSON or table as output format
        #[arg(value_enum, short, long, default_value_t=ListOutputFormat::Table)]
        output: ListOutputFormat,
    },
    ///Delete openDuT resource
    Delete {
        ///Name of openDuT resource
        #[command(subcommand)]
        resource: DeleteResource,
    },
    Config,
}

#[derive(Subcommand)]
enum ListResource {
    ClusterConfigurations(commands::cluster_configuration::list::ListClusterConfigurationsCli),
    ClusterDeployments(commands::cluster_deployment::list::ListClusterDeploymentsCli),
    Peers(commands::peer::list::ListPeersCli),
    Devices(commands::device::list::ListDevicesCli),
    ContainerExecutor(commands::executor::list::ListContainerExecutorCli),
}

#[derive(clap::Args)]
#[group(required=true, multiple = true)]
struct ClusterConfigurationDevices {
    #[arg(long, num_args = 0..)]
    device_names: Vec<DeviceName>,
    #[arg(long, num_args = 0..)]
    device_ids: Vec<String>,
}

#[derive(ValueEnum, Clone)]
pub enum EngineVariants {
    Docker,
    Podman,
}

#[derive(ValueEnum, Clone)]
pub enum NetworkInterfaceType {
    Ethernet,
    Can,
}

#[derive(Subcommand)]
enum CreateResource {
    ClusterConfiguration(commands::cluster_configuration::create::CreateClusterConfigurationCli),
    ClusterDeployment(commands::cluster_deployment::create::CreateClusterDeploymentCli),
    Peer(commands::peer::create::CreatePeerCli),
    ContainerExecutor(commands::executor::create::CreateContainerExecutorCli),
    NetworkInterface(commands::network_interface::create::CreateNetworkInterfaceCli),
    Device(commands::device::create::CreateDeviceCli)
}

#[derive(Subcommand)]
enum DescribeResource {
    ClusterConfiguration(commands::cluster_configuration::describe::DescribeClusterConfigurationCli),
    Peer(commands::peer::describe::DescribePeerCli),
    Device(commands::device::describe::DescribeDeviceCli),
}

#[derive(Subcommand)]
enum FindResource {
    Device(commands::device::find::FindDeviceCli),
}

#[derive(Subcommand)]
enum DeleteResource {
    ClusterConfiguration(commands::cluster_configuration::delete::DeleteClusterConfigurationCli),
    ClusterDeployment(commands::cluster_deployment::delete::DeleteClusterDeploymentCli),
    Peer(commands::peer::delete::DeletePeerCli),
    ContainerExecutor(commands::executor::delete::DeleteContainerExecutorCli),
    NetworkInterface(commands::network_interface::delete::DeleteNetworkInterfaceCli),
    Device(commands::device::delete::DeleteDeviceCli),
}

#[derive(ValueEnum, Clone)]
pub(crate) enum CreateOutputFormat {
    Text,
    Json,
    PrettyJson,
}

#[derive(ValueEnum, Clone)]
enum ListOutputFormat {
    Table,
    Json,
    PrettyJson,
}

#[derive(ValueEnum, Clone)]
enum DescribeOutputFormat {
    Text,
    Json,
    PrettyJson,
}

impl From<CreateOutputFormat> for DescribeOutputFormat {
    fn from(value: CreateOutputFormat) -> Self {
        match value {
            CreateOutputFormat::Text => DescribeOutputFormat::Text,
            CreateOutputFormat::Json => DescribeOutputFormat::Json,
            CreateOutputFormat::PrettyJson => DescribeOutputFormat::PrettyJson,
        }
    }
}

#[derive(ValueEnum, Clone)]
enum DecodePeerSetupOutputFormat {
    Text,
    Json,
    PrettyJson,
}

#[tokio::main]
async fn main() -> ExitCode {
    let red = Style::new().red();
    match execute().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", red.apply_to(error));
            ExitCode::FAILURE
        }
    }
}

async fn execute() -> Result<()> {
    // TODO: make it actually hide secrets in the logging output
    let cleo_config_hide_secrets_override = config::Config::builder()
        .set_override("network.oidc.client.secret", "redacted")
        .map_err(|_error| "Failed to hide cleo secrets.")?
        .build()
        .map_err(|_error| "Failed to hide cleo secrets.")?;

    let settings = load_config("cleo", include_str!("../cleo.toml"), FileFormat::Toml, config::Config::default(), cleo_config_hide_secrets_override)
        .expect("Failed to load config"); // TODO: Point the user to the source of the error.


    let mut carl = {

        let host = settings.config.get_string("network.carl.host")
            .expect("Configuration should contain a valid host name to connect to CARL");

        let port = settings.config.get_int("network.carl.port")
            .expect("Configuration should contain a valid port number to connect to CARL");

        let ca_path = PathBuf::from(settings.config.get_string("network.tls.ca")
            .expect("Configuration should contain a valid path to a CA certificate to connect to CARL"));

        let domain_name_override = settings.config.get_string("network.tls.domain.name.override")
            .expect("Configuration should contain a field for 'domain.name.override'.");
        let domain_name_override = domain_name_override.is_empty().not().then_some(domain_name_override);

        CarlClient::create(host, port as u16, ca_path, domain_name_override, &settings.config)
            .expect("Failed to create CARL client")
    };

    let args = Args::parse();

    match args.command {
        Commands::List { resource, output } => {
            match resource {
                ListResource::ClusterConfigurations(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
                ListResource::ClusterDeployments(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
                ListResource::Peers(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
                ListResource::ContainerExecutor(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
                ListResource::Devices(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
            }
        }
        Commands::Create { resource, output } => {
            match resource {
                CreateResource::ClusterConfiguration(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
                CreateResource::ClusterDeployment(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
                CreateResource::Peer(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
                CreateResource::ContainerExecutor(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
                CreateResource::NetworkInterface(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
                CreateResource::Device(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
            }
        }
        Commands::GeneratePeerSetup(implementation) => {
            implementation.execute(&mut carl).await?;
        }
        Commands::DecodePeerSetup(implementation) => {
            implementation.execute().await?;
        }
        Commands::Describe { resource, output } => {
            match resource {
                DescribeResource::ClusterConfiguration(implementation)=> {
                    implementation.execute(&mut carl, output).await?
                }
                DescribeResource::Peer(implementation)=> {
                    implementation.execute(&mut carl, output).await?
                }
                DescribeResource::Device (implementation)=> {
                    implementation.execute(&mut carl, output).await?
                }
            }
        }
        Commands::Delete { resource} => {
            match resource {
                DeleteResource::ClusterConfiguration(implementation) => {
                    implementation.execute(&mut carl).await?;
                }
                DeleteResource::ClusterDeployment(implementation) => {
                    implementation.execute(&mut carl).await?;
                }
                DeleteResource::Peer(implementation) => {
                    implementation.execute(&mut carl).await?;
                }
                DeleteResource::ContainerExecutor(implementation) => {
                    implementation.execute(&mut carl).await?;
                }
                DeleteResource::NetworkInterface(implementation) => {
                    implementation.execute(&mut carl).await?;
                }
                DeleteResource::Device(implementation) => {
                    implementation.execute(&mut carl).await?;
                }
            }
        }
        Commands::Find { resource, output } => {
            match resource {
                FindResource::Device(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
            }
        }
        Commands::Config => {
            println!("Show cleo configuration: {:?}", settings);
        }
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct ParseablePeerSetup(Box<PeerSetup>);
impl FromStr for ParseablePeerSetup {
    type Err = String;
    fn from_str(string: &str) -> std::result::Result<Self, Self::Err> {
        PeerSetup::decode(string)
            .map(|setup| ParseablePeerSetup(Box::new(setup)))
            .map_err(|error| error.to_string())
    }
}
