use cli_table::{print_stdout, Table, WithTitle};

use opendut_carl_api::carl::CarlClient;
use opendut_model::peer::PeerId;
use opendut_model::viper::{ViperSourceId, ViperTestId, ViperTestName};

use crate::OutputFormat;
use super::SerializableViperTestRun;

/// List all VIPER test run descriptors
#[derive(clap::Parser)]
pub struct ListViperTestRunDescriptorsCli;

#[derive(Table)]
struct ViperTestRunTable {
    #[table(title = "Name")]
    name: ViperTestName,
    #[table(title = "TestID")]
    id: ViperTestId,
    #[table(title = "SourceID")]
    source: ViperSourceId,
    #[table(title = "PeerID")]
    peer: PeerId,
}

impl From<SerializableViperTestRun> for ViperTestRunTable {
    fn from(test: SerializableViperTestRun) -> Self {
        let SerializableViperTestRun { name, id, source, peer, parameters: _ } = test;

        Self { name, id, source, peer }
    }
}

impl ListViperTestRunDescriptorsCli {
    pub async fn execute(self, carl: &mut CarlClient, output: OutputFormat) -> crate::Result<()> {
        let tests = carl.viper.list_viper_test_run_descriptors().await
            .map_err(|error| format!("Could not list VIPER test run descriptors.\n  {error}"))?;

        let serializable_tests = tests.into_iter()
            .map(SerializableViperTestRun::from)
            .collect::<Vec<_>>();

        match output {
            OutputFormat::Table => {
                let test_table = serializable_tests.into_iter()
                    .map(ViperTestRunTable::from)
                    .collect::<Vec<_>>();

                print_stdout(test_table.with_title())
                    .map_err(|error| format!("Could not print VIPER test run descriptors.\n  {error}"))?;
            }
            OutputFormat::Json => {
                let json = serde_json::to_string(&serializable_tests)
                    .map_err(|error| format!("Could not serialize VIPER test run descriptors.\n  {error}"))?;
                println!("{json}");
            }
            OutputFormat::PrettyJson => {
                let json = serde_json::to_string_pretty(&serializable_tests)
                    .map_err(|error| format!("Could not serialize VIPER test run descriptors.\n  {error}"))?;
                println!("{json}");
            }
        }

        Ok(())
    }
}
