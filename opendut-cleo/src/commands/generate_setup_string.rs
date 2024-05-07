use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::PeerId;
use uuid::Uuid;

/// Generate a setup string to setup a peer
#[derive(clap::Parser)]
pub struct GenerateSetupStringCli {
    ///PeerID
    #[arg()]
    id: Uuid,
}

impl GenerateSetupStringCli {
    //TODO: what happens if peer with the ID is already set up?
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let peer_id = PeerId::from(self.id);
        let created_setup = carl
            .peers
            .create_peer_setup(peer_id)
            .await
            .map_err(|error| format!("Could not create setup string.\n  {}", error))?;

        match created_setup.encode() {
            Ok(setup_string) => {
                println!("{}", setup_string);
            }
            Err(_) => {
                println!("Could not configure setup string...")
            }
        }
        Ok(())
    }
}
