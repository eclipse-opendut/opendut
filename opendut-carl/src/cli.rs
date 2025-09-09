use std::collections::HashMap;
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use uuid::Uuid;
use opendut_model::cluster::{ClusterDescriptor, ClusterDeployment, ClusterId};
use opendut_model::peer::{PeerDescriptor, PeerId};
use opendut_model::peer::configuration::{OldPeerConfiguration, PeerConfiguration};
use crate::CreateResult;
use crate::resource::api::Resource;

const BANNER: &str = r"
                         _____     _______
                        |  __ \   |__   __|
   ___  _ __   ___ _ __ | |  | |_   _| |
  / _ \| '_ \ / _ \ '_ \| |  | | | | | |
 | (_) | |_) |  __/ | | | |__| | |_| | |
  \___/| .__/ \___|_| |_|_____/ \__,_|_|
       | |   _____          _____  _
       |_|  / ____|   /\   |  __ \| |
           | |       /  \  | |__) | |
           | |      / /\ \ |  _  /| |
           | |____ / ____ \| | \ \| |____
            \_____/_/    \_\_|  \_\______|

              - He Fixes the Cable -";


#[derive(Parser)]
#[command(name = "opendut-carl")]
#[command(about = "Manage your ECUs via openDuT.")]
#[command(long_version = crate::FORMATTED_VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Default, Subcommand)]
enum Commands {
    /// Launches the CARL Service
    #[default]
    Service,
    /// Access the database directly for administrative purposes
    Db {
        #[command(subcommand)]
        db_command: DbCommand,
    },
}

#[derive(Subcommand)]
enum DbCommand {
    /// List resources in the database
    List {
        #[arg()]
        resource_kind: ResourceKind,
    },
    /// Remove a resource with the given ID
    Delete {
        #[arg()]
        resource_kind: ResourceKind,
        /// ID of the resource to delete
        #[arg()]
        id: Uuid,
    }
}
#[derive(Clone, ValueEnum)]
enum ResourceKind {
    ClusterDescriptor,
    ClusterDeployment,
    OldPeerConfiguration,
    PeerConfiguration,
    PeerDescriptor,
}

pub async fn cli() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command.unwrap_or_default() {
        Commands::Service => {
            println!("{BANNER}\n{version_info}", version_info=crate::FORMATTED_VERSION);
            crate::create(
                opendut_util::settings::Config::default(),
                crate::StartupOptions::default()
            ).await?;
        },
        Commands::Db { db_command } => {
            let CreateResult::ResourceManagerRef(resource_manager)
                = crate::create(
                    opendut_util::settings::Config::default(),
                    crate::StartupOptions {
                        telemetry_enabled: false,
                        return_resource_manager_ref: true,
                    }
                ).await?
            else {
                panic!("Expected ResourceManagerRef during CARL CLI startup.")
            };

            match db_command {
                DbCommand::List { resource_kind } => {
                    fn print<R: Resource + Serialize>(values: HashMap<R::Id, R>) -> anyhow::Result<()> {
                        let values = values.values().collect::<Vec<_>>();
                        let string = serde_json::to_string(&values)?;
                        println!("{string}");
                        Ok(())
                    }

                    match resource_kind {
                        ResourceKind::ClusterDescriptor => print(resource_manager.list::<ClusterDescriptor>().await?),
                        ResourceKind::ClusterDeployment => print(resource_manager.list::<ClusterDeployment>().await?),
                        ResourceKind::OldPeerConfiguration => print(resource_manager.list::<OldPeerConfiguration>().await?),
                        ResourceKind::PeerConfiguration => print(resource_manager.list::<PeerConfiguration>().await?),
                        ResourceKind::PeerDescriptor => print(resource_manager.list::<PeerDescriptor>().await?),
                    }?;
                }
                DbCommand::Delete { resource_kind, id } => {
                    match resource_kind {
                        ResourceKind::ClusterDescriptor => {
                            resource_manager.remove::<ClusterDescriptor>(ClusterId::from(id)).await?;
                        }
                        ResourceKind::ClusterDeployment => {
                            resource_manager.remove::<ClusterDeployment>(ClusterId::from(id)).await?;
                        }
                        ResourceKind::OldPeerConfiguration => {
                            resource_manager.remove::<OldPeerConfiguration>(PeerId::from(id)).await?;
                        }
                        ResourceKind::PeerConfiguration => {
                            resource_manager.remove::<PeerConfiguration>(PeerId::from(id)).await?;
                        }
                        ResourceKind::PeerDescriptor => {
                            resource_manager.remove::<PeerDescriptor>(PeerId::from(id)).await?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
