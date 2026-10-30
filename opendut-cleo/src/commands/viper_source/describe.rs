use indoc::formatdoc;

use opendut_carl_api::carl::CarlClient;
use opendut_model::viper::{ViperSourceDescriptor, ViperSourceId};

use crate::OutputFormat;
use super::SerializableViperSource;

/// Describe a VIPER source descriptor
#[derive(clap::Parser)]
pub struct DescribeViperSourceDescriptorCli {
    /// ID of the source
    #[arg()]
    id: ViperSourceId,
}

impl DescribeViperSourceDescriptorCli {
    pub async fn execute(self, carl: &mut CarlClient, output: OutputFormat) -> crate::Result<()> {
        let source_id = self.id;

        let descriptor = carl.viper.get_viper_source_descriptor(source_id).await
            .map_err(|error| format!("Failed to retrieve VIPER source descriptor <{source_id}>.\n  {error}"))?;

        render_viper_source_descriptor(descriptor, output)
    }
}

pub fn render_viper_source_descriptor(descriptor: ViperSourceDescriptor, output: OutputFormat) -> crate::Result<()> {
    let source = SerializableViperSource::from(descriptor);

    let text = match output {
        OutputFormat::Table => {
            let id = source.id;
            let name = source.name;
            let url = source.url;
            let kind = source.kind;
            
            formatdoc!("
                VIPER Source: {name}
                  Id: {id}
                  URL: {url}
                  Kind: {kind}\
            ")
        }
        OutputFormat::Json => {
            serde_json::to_string(&source)
                .map_err(|error| format!("Could not serialize VIPER source.\n  {error}"))?
        }
        OutputFormat::PrettyJson => {
            serde_json::to_string_pretty(&source)
                .map_err(|error| format!("Could not serialize VIPER source.\n  {error}"))?
        }
    };

    println!("{text}");
    Ok(())
}
