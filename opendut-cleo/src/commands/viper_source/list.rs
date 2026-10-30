use cli_table::{print_stdout, Table, WithTitle};
use url::Url;

use opendut_carl_api::carl::CarlClient;
use opendut_model::viper::ViperSourceId;

use crate::OutputFormat;
use super::SerializableViperSource;

/// List all VIPER source descriptors
#[derive(clap::Parser)]
pub struct ListViperSourceDescriptorsCli;

#[derive(Table)]
struct ViperSourceTable {
    #[table(title = "Name")]
    name: String,
    #[table(title = "SourceID")]
    id: ViperSourceId,
    #[table(title = "URL")]
    url: Url,
    #[table(title = "Kind")]
    kind: &'static str,
}

impl From<SerializableViperSource> for ViperSourceTable {
    fn from(source: SerializableViperSource) -> Self {
        let SerializableViperSource { name, id, url, kind } = source;

        Self { name, id, url, kind }
    }
}

impl ListViperSourceDescriptorsCli {
    pub async fn execute(self, carl: &mut CarlClient, output: OutputFormat) -> crate::Result<()> {
        let sources = carl.viper.list_viper_source_descriptors().await
            .map_err(|error| format!("Could not list VIPER source descriptors.\n  {error}"))?;

        let serializable_sources = sources.into_iter()
            .map(SerializableViperSource::from)
            .collect::<Vec<_>>();

        match output {
            OutputFormat::Table => {
                let source_table = serializable_sources.into_iter()
                    .map(ViperSourceTable::from)
                    .collect::<Vec<_>>();

                print_stdout(source_table.with_title())
                    .map_err(|error| format!("Could not print VIPER source descriptors.\n  {error}"))?;
            }
            OutputFormat::Json => {
                let json = serde_json::to_string(&serializable_sources)
                    .map_err(|error| format!("Could not serialize VIPER source descriptors.\n  {error}"))?;
                println!("{json}");
            }
            OutputFormat::PrettyJson => {
                let json = serde_json::to_string_pretty(&serializable_sources)
                    .map_err(|error| format!("Could not serialize VIPER source descriptors.\n  {error}"))?;
                println!("{json}");
            }
        }

        Ok(())
    }
}
