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
    pub async fn execute(self, carl: &mut CarlClient, cleo_oidc_client_id: String,) -> crate::Result<()> {
        let peer_id = PeerId::from(self.id);
        let created_setup = carl
            .peers
            .create_peer_setup(peer_id, cleo_oidc_client_id)
            .await
            .map_err(|error| format!("Could not create setup string.\n  {}", error))?;

        match created_setup.encode() {
            Ok(setup_string) => {
                println!("{}", setup_string);
                eprintln!("Setup-Strings may only be used to set up one host. For setting up multiple hosts, you should create a peer for each host.");
            }
            Err(_) => {
                println!("Could not configure setup string...")
            }
        }
        Ok(())
    }
}
