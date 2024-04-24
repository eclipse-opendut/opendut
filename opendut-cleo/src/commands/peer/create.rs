use console::Style;
use uuid::Uuid;

use crate::{CreateOutputFormat};
use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName};
use opendut_types::peer::executor::{ExecutorDescriptors};

pub async fn execute(
    carl: &mut CarlClient,
    name: String,
    id: Option<Uuid>,
    location: Option<String>,
    output: CreateOutputFormat,
) -> crate::Result<()> {
    let id = PeerId::from(id.unwrap_or_else(Uuid::new_v4));

    let name = PeerName::try_from(name)
        .map_err(|error| format!("Could not create peer.\n  {}", error))?;

    let location = location
        .map(PeerLocation::try_from)
        .transpose()
        .map_err(|error| format!("Could not create peer.\n  {}", error))?;

    let descriptor: PeerDescriptor = PeerDescriptor {
        id,
        name: Clone::clone(&name),
        location,
        network_configuration: Default::default(),
        topology: Default::default(),
        executors: ExecutorDescriptors {
            executors: vec![],
        }
    };
    carl.peers
        .store_peer_descriptor(descriptor.clone())
        .await
        .map_err(|error| format!("Failed to create new peer.\n  {error}"))?;
    let bold = Style::new().bold();
    match output {
        CreateOutputFormat::Text => {
            println!(
                "Created the peer '{}' with the ID: <{}>",
                name,
                bold.apply_to(id)
            );
        }
        CreateOutputFormat::Json => {
            let json = serde_json::to_string(&descriptor).unwrap();
            println!("{}", json);
        }
        CreateOutputFormat::PrettyJson => {
            let json = serde_json::to_string_pretty(&descriptor).unwrap();
            println!("{}", json);
        }
    }
    Ok(())
}