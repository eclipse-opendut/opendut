use indoc::indoc;

use opendut_carl_api::carl::CarlClient;
use opendut_model::viper::{ViperTestId, ViperTestRunDescriptor};

use crate::OutputFormat;
use super::SerializableViperTestRun;

/// Describe a VIPER test run descriptor
#[derive(clap::Parser)]
pub struct DescribeViperTestRunDescriptorCli {
    /// ID of the test
    #[arg()]
    id: ViperTestId,
}

impl DescribeViperTestRunDescriptorCli {
    pub async fn execute(self, carl: &mut CarlClient, output: OutputFormat) -> crate::Result<()> {
        let test_id = self.id;

        let descriptor = carl.viper.get_viper_test_run_descriptor(test_id).await
            .map_err(|error| format!("Failed to retrieve VIPER test run descriptor <{test_id}>.\n  {error}"))?;

        render_viper_test_run_descriptor(descriptor, output)
    }
}

pub fn render_viper_test_run_descriptor(descriptor: ViperTestRunDescriptor, output: OutputFormat) -> crate::Result<()> {
    let test = SerializableViperTestRun::from(descriptor);

    let text = match output {
        OutputFormat::Table => {
            let parameters = test.parameters.iter()
                .map(|(name, value)| {
                    let value = value.as_ref().map(|value| value.to_string())
                        .unwrap_or_else(|| String::from("<default>"));
                    format!("{name}={value}")
                })
                .collect::<Vec<_>>()
                .join(", ");

            format!(indoc!("
                VIPER Test Run: {}
                  Id: {}
                  Source: {}
                  Peer: {}
                  Parameters: [{}]"), test.name, test.id, test.source, test.peer, parameters)
        }
        OutputFormat::Json => {
            serde_json::to_string(&test)
                .map_err(|error| format!("Could not serialize VIPER test run descriptor.\n  {error}"))?
        }
        OutputFormat::PrettyJson => {
            serde_json::to_string_pretty(&test)
                .map_err(|error| format!("Could not serialize VIPER test run descriptor.\n  {error}"))?
        }
    };

    println!("{text}");
    Ok(())
}
