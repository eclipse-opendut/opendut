use std::ops::Not;
use std::path::PathBuf;
use std::process::ExitCode;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use console::Style;
use uuid::Uuid;
use opendut_carl_api::carl::{CaCertInfo, CarlClient};
use opendut_carl_api::carl::metadata::version_compatibility::VersionCompatibilityInfo;
use opendut_types::topology::{DeviceId, DeviceName};
use opendut_util::settings::{load_config, FileFormat, LoadedConfig};

mod commands;
pub mod parse;

type Error = String;
type Result<T> = std::result::Result<T, Error>;

shadow_rs::shadow!(app_info);
shadow_formatted_version::from_shadow!(app_info);

/// CLEO is a command line tool to manage openDuT resources.
#[derive(Parser)]
#[command(author, about, long_about = None)]
#[command(long_version = crate::FORMATTED_VERSION)]
struct Args {
    #[command(subcommand)]
    command: Commands,
    /// Enable more detailed logging.
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    ///Authenticate CLEO with setup string
    Setup(commands::setup::SetupCli),
    ///Display openDuT resources
    List {
        #[command(subcommand)]
        resource: ListResource,
        ///JSON, prettified JSON or table as output format
        #[arg(global = true, value_enum, short, long, default_value_t=ListOutputFormat::Table)]
        output: ListOutputFormat,
    },
    /// Create openDuT resources from specifications files
    Apply(commands::apply::ApplyCli),
    ///Create openDuT resource
    Create {
        #[command(subcommand)]
        resource: Box<CreateResource>,
        ///Text, JSON or prettified JSON as output format
        #[arg(global = true, value_enum, short, long, default_value_t=CreateOutputFormat::Text)]
        output: CreateOutputFormat,
    },
    GenerateSetupString(commands::generate_setup_string::GenerateSetupStringCli),
    DecodeSetupString(commands::decode_setup_string::DecodeSetupStringCli),
    ///Describe openDuT resource
    Describe {
        ///Name of openDuT resource
        #[command(subcommand)]
        resource: DescribeResource,
        ///JSON, prettified JSON or table as output format
        #[arg(global = true, value_enum, short, long, default_value_t=DescribeOutputFormat::Text)]
        output: DescribeOutputFormat,
    },
    ///Find openDuT resource
    Find {
        ///Name of openDuT resource
        #[command(subcommand)]
        resource: FindResource,
        ///JSON, prettified JSON or table as output format
        #[arg(global = true, value_enum, short, long, default_value_t=ListOutputFormat::Table)]
        output: ListOutputFormat,
    },
    ///Delete openDuT resource
    Delete {
        ///Name of openDuT resource
        #[command(subcommand)]
        resource: DeleteResource,
    },
    /// Wait for a resource
    Await {
        #[command(subcommand)]
        resource: AwaitResource,
    },
    ///Show the configuration that CLEO currently uses
    Config,
    /// Generates shell completion
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell
    },
}

#[derive(Subcommand)]
enum ListResource {
    #[command(alias="cluster-descriptor", alias="cluster-configurations")]
    ClusterDescriptors(commands::cluster_descriptor::list::ListClusterDescriptorsCli),
    #[command(alias="cluster-deployment")]
    ClusterDeployments(commands::cluster_deployment::list::ListClusterDeploymentsCli),
    #[command(alias="peer")]
    Peers(commands::peer::list::ListPeersCli),
    #[command(alias="device")]
    Devices(commands::device::list::ListDevicesCli),
    #[command(alias="container-executor")]
    ContainerExecutors(commands::executor::list::ListContainerExecutorCli),
}

#[derive(clap::Args)]
#[group(required=true, multiple = true)]
struct ClusterDescriptorDevices {
    #[arg(long, num_args = 0..)]
    device_names: Vec<DeviceName>,
    #[arg(long, num_args = 0..)]
    device_ids: Vec<DeviceId>,
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
    ClusterDeployment(commands::cluster_deployment::create::CreateClusterDeploymentCli),
    #[command(alias="cluster-configuration")]
    ClusterDescriptor(commands::cluster_descriptor::create::CreateClusterDescriptorCli),
    Peer(commands::peer::create::CreatePeerCli),
    ContainerExecutor(commands::executor::create::CreateContainerExecutorCli),
    NetworkInterface(commands::network_interface::create::CreateNetworkInterfaceCli),
    Device(commands::device::create::CreateDeviceCli),
    /// Generate a random UUID, which can be used for assigning a new ID to a resource
    Uuid,
}

#[derive(Subcommand)]
enum DescribeResource {
    #[command(alias="cluster-configuration")]
    ClusterDescriptor(commands::cluster_descriptor::describe::DescribeClusterDescriptorCli),
    Peer(commands::peer::describe::DescribePeerCli),
    Device(commands::device::describe::DescribeDeviceCli),
}

#[derive(Subcommand)]
enum FindResource {
    Device(commands::device::find::FindDeviceCli),
}

#[derive(Subcommand)]
enum DeleteResource {
    ClusterDeployment(commands::cluster_deployment::delete::DeleteClusterDeploymentCli),
    #[command(alias="cluster-configuration")]
    ClusterDescriptor(commands::cluster_descriptor::delete::DeleteClusterDescriptorCli),
    Peer(commands::peer::delete::DeletePeerCli),
    ContainerExecutor(commands::executor::delete::DeleteContainerExecutorCli),
    NetworkInterface(commands::network_interface::delete::DeleteNetworkInterfaceCli),
    Device(commands::device::delete::DeleteDeviceCli),
}

#[derive(Subcommand)]
enum AwaitResource {
    PeerOnline(commands::wait::peer_online::WaitPeerOnlineCli),
    ClusterPeersOnline(commands::wait::cluster_peers_online::WaitPeersInClusterOnline),
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
enum DecodeSetupStringOutputFormat {
    Text,
    Json,
    PrettyJson,
}

#[tokio::main]
async fn main() -> ExitCode {
    let result = execute().await;
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let red = Style::new().red();
            eprintln!("{}", red.apply_to(error));
            ExitCode::FAILURE
        }
    }
}

async fn execute() -> Result<()> {
    let cleo_config_hide_secrets_override = config::Config::builder()
        .set_override("network.oidc.client.secret", "redacted")
        .map_err(|_| "Failed to hide CLEO secrets.")?
        .build()
        .map_err(|_| "Failed to hide CLEO secrets.")?;

    let settings = load_config("cleo", include_str!("../cleo.toml"), FileFormat::Toml, config::Config::default(), cleo_config_hide_secrets_override)
        .expect("Failed to load config"); // TODO: Point the user to the source of the error.

    let args = Args::parse();

    let telemetry_shutdown_handle = if args.verbose {
        use opendut_util::telemetry;

        let config = telemetry::logging::LoggingConfig::load(&settings.config)
            .map_err(|error| format!("Error while loading logging configuration: {error}"))?;

        let shutdown_handle = telemetry::initialize_with_config(
            config,
            telemetry::opentelemetry_types::Opentelemetry::Disabled,
        ).await
            .map_err(|_| "Error while initializing logging.")?;

        Some(shutdown_handle)
    } else {
        None
    };

    execute_command(args.command, &settings).await?;

    if let Some(mut shutdown_handle) = telemetry_shutdown_handle {
        shutdown_handle.shutdown();
    }
    Ok(())
}

async fn execute_command(commands: Commands, settings: &LoadedConfig) -> Result<()>{
    match commands {
        Commands::Setup(implementation) => {
            implementation.execute().await?;
        },
        Commands::List { resource, output } => {
            let mut carl = create_carl_client(&settings.config).await;
            match resource {
                ListResource::ClusterDescriptors(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
                ListResource::ClusterDeployments(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
                ListResource::Peers(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
                ListResource::ContainerExecutors(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
                ListResource::Devices(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
            }
        }
        Commands::Apply(implementation) => {
            let mut carl = create_carl_client(&settings.config).await;
            implementation.execute(&mut carl).await?;
        }
        Commands::Create { resource, output } => {
            match *resource {
                CreateResource::ClusterDescriptor(implementation) => {
                    let mut carl = create_carl_client(&settings.config).await;
                    implementation.execute(&mut carl, output).await?;
                }
                CreateResource::ClusterDeployment(implementation) => {
                    let mut carl = create_carl_client(&settings.config).await;
                    implementation.execute(&mut carl, output).await?;
                }
                CreateResource::Peer(implementation) => {
                    let mut carl = create_carl_client(&settings.config).await;
                    implementation.execute(&mut carl, output).await?;
                }
                CreateResource::ContainerExecutor(implementation) => {
                    let mut carl = create_carl_client(&settings.config).await;
                    implementation.execute(&mut carl, output).await?;
                }
                CreateResource::NetworkInterface(implementation) => {
                    let mut carl = create_carl_client(&settings.config).await;
                    implementation.execute(&mut carl, output).await?;
                }
                CreateResource::Device(implementation) => {
                    let mut carl = create_carl_client(&settings.config).await;
                    implementation.execute(&mut carl, output).await?;
                }
                CreateResource::Uuid => {
                    println!("{}", Uuid::new_v4());
                }
            }
        }
        Commands::GenerateSetupString(implementation) => {
            let mut carl = create_carl_client(&settings.config).await;
            let cleo_oidc_client_id = get_cleo_oidc_client_id(&settings.config).await;
            implementation.execute(&mut carl, cleo_oidc_client_id).await?;
        }
        Commands::DecodeSetupString(implementation) => {
            implementation.execute().await?;
        }
        Commands::Describe { resource, output } => {
            let mut carl = create_carl_client(&settings.config).await;
            match resource {
                DescribeResource::ClusterDescriptor(implementation)=> {
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
            let mut carl = create_carl_client(&settings.config).await;
            match resource {
                DeleteResource::ClusterDescriptor(implementation) => {
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
            let mut carl = create_carl_client(&settings.config).await;
            match resource {
                FindResource::Device(implementation) => {
                    implementation.execute(&mut carl, output).await?;
                }
            }
        }
        Commands::Config => {
            println!("Active CLEO configuration: {settings:?}");
        }
        Commands::Completions { shell } => {
            let mut cmd = Args::command();
            commands::completions::print_completions(shell, &mut cmd);
        }
        Commands::Await { resource } => {
            let mut carl = create_carl_client(&settings.config).await;
            match resource {
                AwaitResource::PeerOnline(cli) => {
                    cli.execute(&mut carl).await?;
                }
                AwaitResource::ClusterPeersOnline(cli) => {
                    cli.execute(&mut carl).await?;
                }
            };
        }
    }
    Ok(())
}


pub async fn create_carl_client(config: &config::Config) -> CarlClient {
    let host = config.get_string("network.carl.host")
        .expect("Configuration should contain a valid host name to connect to CARL");

    let port = config.get_int("network.carl.port")
        .expect("Configuration should contain a valid port number to connect to CARL");

    let ca_cert_info = match config.get_string("network.tls.ca.content") {
        Ok(content_string) => CaCertInfo::Content(content_string),
        Err(_) => {
            let path = config.get_string("network.tls.ca")
                .expect("Configuration should contain a valid path to a CA certificate to connect to CARL.");
            CaCertInfo::Path(PathBuf::from(path))
        },
    };

    let domain_name_override = config.get_string("network.tls.domain.name.override")
        .expect("Configuration should contain a field for 'domain.name.override'.");
    let domain_name_override = domain_name_override.is_empty().not().then_some(domain_name_override);

    let mut carl_client = CarlClient::create(host, port as u16, &ca_cert_info, &domain_name_override, config).await
        .expect("Failed to create CARL client");

    let _ignore_errors = opendut_carl_api::carl::metadata::version_compatibility::log_version_compatibility_with_carl(
        VersionCompatibilityInfo {
            own_version: app_info::PKG_VERSION,
            own_name: String::from("CLEO"),
            upgrade_hint: Some(String::from(
                "You can update CLEO by downloading the newest version from the openDuT web-UI."
            )),
        },
        &mut carl_client,
    ).await;

    carl_client
}

pub async fn get_cleo_oidc_client_id(config: &config::Config) -> String {
    match config.get_string("network.oidc.client.id") {
        Ok(id) => { id }
        Err(_) => { String::from("cleoCli") }
    }
}
