use clap::{Args, Subcommand};
use uuid::Uuid;
use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::PeerId;
use opendut_types::topology::{AccessoryDescription, AccessoryDescriptor, AccessoryId, AccessoryModel, AccessoryName};
use crate::{CreateOutputFormat, DescribeOutputFormat};

/// Create a device
#[derive(clap::Parser)]
pub struct CreateAccessoryCli {
    #[command(subcommand)]
    command: AccessoryCommands,
}

#[derive(Subcommand)]
enum AccessoryCommands {
    MansonHcs3304 {
        #[command(flatten)]
        common: CommonArgs,
        #[arg(short, long)]
        serial_port: String,
    },
}

#[derive(Args)]
struct CommonArgs {
    ///ID of the peer to add the accessory to
    #[arg(long)]
    peer_id: Uuid,
    ///ID of the accessory to be added or updated
    #[arg(long)]
    accessory_id: Option<Uuid>,
    ///Name of the accessory
    #[arg(long)]
    name: String,
    ///Description of accessory
    #[arg(long)]
    description: Option<String>,
}



impl CreateAccessoryCli {
    #[allow(clippy::too_many_arguments)]
    pub async fn execute(self, carl: &mut CarlClient, output: CreateOutputFormat) -> crate::Result<()> {
        let common_args = match self.command {
            AccessoryCommands::MansonHcs3304 { common, serial_port: _ } => common,
        };
        let peer_id = PeerId::from(common_args.peer_id);
        let accessory_id = common_args.accessory_id.map(AccessoryId::from).unwrap_or(AccessoryId::random());

        let mut peer_descriptor = carl.peers.get_peer_descriptor(peer_id).await
            .map_err(|_| format!("Failed to get peer with ID <{}>.", peer_id))?;

        let accessory_model = match self.command {
            AccessoryCommands::MansonHcs3304 { common: _, serial_port } => {
                AccessoryModel::MansonHcs3304 { serial_port: serial_port.clone() }
            },
        };

        if peer_descriptor.topology.accessories.iter().any(|accessory| accessory.id == accessory_id) {
            Err(String::from("Cannot create new accessory because an accessory with this ID already exists."))?;
        }

        peer_descriptor.topology.accessories.push(
            AccessoryDescriptor {
                id: accessory_id,
                name: AccessoryName::try_from(common_args.name)
                    .map_err(|error| error.to_string())?,
                description: common_args.description
                    .map(AccessoryDescription::try_from)
                    .transpose()
                    .map_err(|error| error.to_string())?,
                model: accessory_model,
            }
        );

        carl.peers.store_peer_descriptor(Clone::clone(&peer_descriptor)).await
            .map_err(|error| format!("Failed to update peer <{}>.\n  {}", peer_id, error))?;
        let output_format = DescribeOutputFormat::from(output);
        crate::commands::peer::describe::render_peer_descriptor(peer_descriptor, output_format);

        Ok(())
    }
}
