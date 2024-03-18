use std::ops::Not;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{Parser, Subcommand, ValueEnum};
use console::Style;
use uuid::Uuid;

use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::PeerSetup;
use opendut_types::peer::executor::{ContainerCommand, ContainerCommandArgument, ContainerDevice, ContainerImage, ContainerName, ContainerPortSpec, ContainerVolume};
use opendut_types::topology::DeviceName;
use opendut_types::util::net::NetworkInterfaceName;
use opendut_util::settings::{FileFormat, load_config};

mod commands;

type Error = String;
type Result<T> = std::result::Result<T, Error>;

opendut_util::app_info!();

/// CLEO is a command line tool to manage openDuT resources.
#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
#[command(long_version = crate::app_info::formatted())]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
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
    ///Generate a setup string
    GeneratePeerSetup {
        ///PeerID
        #[arg(short, long)]
        id: Uuid,
    },
    DecodePeerSetup {
        ///Setup string
        #[arg()]
        setup_string: ParseablePeerSetup,
        ///Text, JSON or prettified JSON as output format
        #[arg(value_enum, short, long, default_value_t=DecodePeerSetupOutputFormat::Json)]
        output: DecodePeerSetupOutputFormat,
    },
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

#[derive(Subcommand, Clone, Debug)]
enum ListResource {
    ClusterConfigurations,
    ClusterDeployments,
    Peers,
    Devices,
    ContainerExecutor {
        ///PeerID
        #[arg(short, long)]
        id: Uuid,
    },
}

#[derive(Debug, Clone, clap::Args)]
#[group(required=true, multiple = true)]
struct ClusterConfigurationDevices {
    #[arg(long, num_args = 0..)]
    device_names: Vec<DeviceName>,
    #[arg(long, num_args = 0..)]
    device_ids: Vec<String>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum EngineVariants {
    Docker,
    Podman,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum NetworkInterfaceType {
    Ethernet,
    Can,
}

#[derive(Subcommand, Clone, Debug)]
enum CreateResource {
    ClusterConfiguration {
        ///Name of the cluster
        #[arg(short, long)]
        name: String,
        ///ClusterID
        #[arg(short, long)]
        cluster_id: Option<Uuid>,
        ///PeerID of the leader
        #[arg(short, long)]
        leader_id: Uuid,
        ///List of devices in cluster
        #[clap(flatten)]
        devices: ClusterConfigurationDevices,
    },
    ClusterDeployment {
        ///ClusterID
        #[arg(short, long)]
        id: Uuid,
    },
    Peer {
        ///Name of peer
        #[arg(short, long)]
        name: String,
        ///PeerID
        #[arg(short, long)]
        id: Option<Uuid>,
        ///Location of peer
        #[arg(long)]
        location: Option<String>,
    },
    ContainerExecutor {
        ///ID of the peer to add the container executor to
        #[arg(long)]
        peer_id: Uuid,
        ///Engine
        #[arg(short, long)]
        engine: EngineVariants,
        ///Container name
        #[arg(short, long)]
        name: Option<ContainerName>,
        ///Container image
        #[arg(short, long)]
        image: ContainerImage,
        ///Container volumes
        #[arg(short, long, num_args = 1..)]
        volumes: Option<Vec<ContainerVolume>>,
        ///Container devices
        #[arg(long, num_args = 1..)]
        devices: Option<Vec<ContainerDevice>>,
        ///Container envs
        #[arg(long, num_args = 1..)]
        envs: Option<Vec<String>>,
        ///Container ports
        #[arg(short, long, num_args = 1..)]
        ports: Option<Vec<ContainerPortSpec>>,
        ///Container command
        #[arg(short, long)]
        command: Option<ContainerCommand>,
        ///Container arguments
        #[arg(short, long, num_args = 1..)]
        args: Option<Vec<ContainerCommandArgument>>,
    },
    NetworkInterface {
        ///ID of the peer to add the network interface to
        #[arg(long)]
        peer_id: Uuid,
        ///Type of the network interface
        #[arg(long("type"))]
        interface_type: NetworkInterfaceType,
        ///Name of the network interface
        #[arg(long("name"))]
        interface_name: String,
    },
    Device {
        ///ID of the peer to add the device to
        #[arg(long)]
        peer_id: Uuid,
        ///ID of the device to be added or updated
        #[arg(long)]
        device_id: Option<Uuid>,
        ///Name of the device
        #[arg(long)]
        name: Option<String>,
        ///Description of device
        #[arg(long)]
        description: Option<String>,
        ///Interface of device
        #[arg(long)]
        interface: Option<NetworkInterfaceName>,
        /// Tags of device
        #[arg(long("tag"))]
        tags: Option<Vec<String>>,
    }
}

#[derive(Subcommand, Clone, Debug)]
enum DescribeResource {
    ClusterConfiguration {
        ///ClusterID
        #[arg(short, long)]
        id: Uuid,
    },
    Peer {
        ///PeerID
        #[arg(short, long)]
        id: Uuid,
    },
    Device {
        ///DeviceID
        #[arg(short, long)]
        id: Uuid,
    },
}

#[derive(Subcommand, Clone, Debug)]
enum FindResource {
    Device {
        ///Criteria for search
        #[arg(required = true, value_delimiter = ' ', num_args = 1..)]
        criteria: Vec<String>,
    },
}

#[derive(Subcommand, Clone, Debug)]
enum DeleteResource {
    ClusterConfiguration {
        ///ClusterID
        #[arg(short, long)]
        id: Uuid,
    },
    ClusterDeployment {
        ///ClusterID
        #[arg(short, long)]
        id: Uuid,
    },
    Peer {
        ///PeerID
        #[arg(short, long)]
        id: Uuid,
    },
    ContainerExecutor {
        ///ID of the peer to delete the container executor from
        #[arg(long)]
        peer_id: Uuid,
        ///Container images to delete
        #[arg(short, long)]
        images: Vec<ContainerImage>,
    },
    NetworkInterface {
        ///ID of the peer to delete the network configuration from
        #[arg(long)]
        peer_id: Uuid,
        ///NetworkConfiguration Interface
        #[arg(long("interface"))]
        interfaces: Vec<String>,
    },
    Device {
        ///DeviceID
        #[arg(short, long)]
        id: Uuid,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub(crate) enum CreateOutputFormat {
    Text,
    Json,
    PrettyJson,
}

#[derive(ValueEnum, Clone, Debug)]
enum ListOutputFormat {
    Table,
    Json,
    PrettyJson,
}

#[derive(ValueEnum, Clone, Debug)]
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

#[derive(ValueEnum, Clone, Debug)]
enum DecodePeerSetupOutputFormat {
    Text,
    Json,
    PrettyJson,
}



#[tokio::main]
async fn main() {
    let red = Style::new().red();
    if let Err(error) = execute().await {
        eprintln!("{}", red.apply_to(error));
        // indicate error to calling processes by setting exit code to 1
        std::process::exit(1);
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
                ListResource::ClusterConfigurations => {
                    commands::cluster_configuration::list::execute(&mut carl, output).await?;
                }
                ListResource::ClusterDeployments => {
                    commands::cluster_deployment::list::execute(&mut carl, output).await?;
                }
                ListResource::Peers => {
                    commands::peer::list::execute(&mut carl, output).await?;
                }
                ListResource::ContainerExecutor{ id }  => {
                    commands::executor::list::execute(&mut carl, id, output).await?;
                }
                ListResource::Devices => {
                    commands::device::list_devices(&mut carl, output).await?;
                }
            }
        }
        Commands::Create { resource, output } => {
            match resource {
                CreateResource::ClusterConfiguration { name, cluster_id, leader_id, devices  } => {
                    commands::cluster_configuration::create::execute(&mut carl, name, cluster_id, leader_id, devices.device_names, devices.device_ids, output).await?;
                }
                CreateResource::ClusterDeployment { id} => {
                    commands::cluster_deployment::create::execute(&mut carl, id, output).await?;
                }
                CreateResource::Peer { name, id, location} => {
                    commands::peer::create::execute(&mut carl, name, id, location, output).await?;
                }
                CreateResource::ContainerExecutor { peer_id, engine, name, image, volumes, devices, envs, ports, command, args} => {
                    commands::executor::create::execute(&mut carl, peer_id, engine, name, image, volumes, devices, envs, ports, command, args, output).await?;
                }
                CreateResource::NetworkInterface { peer_id, interface_type, interface_name} => {
                    commands::network_interface::create::execute(&mut carl, peer_id, interface_type, interface_name, output).await?;
                }
                CreateResource::Device { peer_id, device_id, name, description, interface, tags } => {
                    commands::device::create::execute(&mut carl, peer_id, device_id, name, description, interface, tags, output).await?;
                }
            }
        }
        Commands::GeneratePeerSetup { id } => {
            commands::peer::generate_peer_setup::execute(&mut carl, id).await?;
        }
        Commands::DecodePeerSetup { setup_string, output } => {
            commands::peer::decode_peer_setup::execute(*setup_string.0, output).await?;
        }
        Commands::Describe { resource, output } => {
            match resource {
                DescribeResource::ClusterConfiguration { id } => {
                    commands::cluster_configuration::describe::execute(&mut carl, id, output).await?
                }
                DescribeResource::Peer { id } => {
                    commands::peer::describe::execute(&mut carl, id, output).await?
                }
                DescribeResource::Device { id } => {
                    commands::device::describe::execute(&mut carl, id, output).await?
                }
            }
        }
        Commands::Delete { resource} => {
            match resource {
                DeleteResource::ClusterConfiguration { id } => {
                    commands::cluster_configuration::delete::execute(&mut carl, id).await?;
                }
                DeleteResource::ClusterDeployment { id } => {
                    commands::cluster_deployment::delete::execute(&mut carl, id).await?;
                }
                DeleteResource::Peer { id } => {
                    commands::peer::delete::execute(&mut carl, id).await?;
                }
                DeleteResource::ContainerExecutor { peer_id, images} => {
                    commands::executor::delete::execute(&mut carl, peer_id, images).await?;
                }
                DeleteResource::NetworkInterface { peer_id,  interfaces} => {
                    commands::network_interface::delete::execute(&mut carl, peer_id, interfaces).await?;
                }
                DeleteResource::Device { id } => {
                    commands::device::delete::execute(&mut carl, id).await?;
                }
            }
        }
        Commands::Find { resource, output } => {
            match resource {
                FindResource::Device { criteria } => {
                    commands::device::find::execute(&mut carl, criteria, output).await?;
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
