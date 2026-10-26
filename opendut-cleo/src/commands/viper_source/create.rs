use opendut_carl_api::carl::CarlClient;
use opendut_model::viper::{ViperSourceDescriptor, ViperSourceId, ViperSourceKind, ViperTestSuiteIdentifier};
use url::Url;

use crate::OutputFormat;
use super::SerializableViperSource;

/// Create a VIPER source descriptor
#[derive(clap::Parser)]
pub struct CreateViperSourceDescriptorCli {
    /// Name of the test suite in the source
    #[arg(short, long, value_parser = parse_suite_identifier)]
    name: ViperTestSuiteIdentifier,
    /// URL of the source
    #[arg(short, long)]
    url: Url,
    /// Source kind
    #[arg(short, long, value_enum)]
    kind: SourceKind,
    /// ID of the source; generates a random UUID when omitted
    #[arg(short, long)]
    id: Option<ViperSourceId>,
}

#[derive(Clone, clap::ValueEnum)]
enum SourceKind {
    Http,
    Git,
}

impl From<SourceKind> for ViperSourceKind {
    fn from(kind: SourceKind) -> Self {
        match kind {
            SourceKind::Http => Self::Http,
            SourceKind::Git => Self::Git,
        }
    }
}

fn parse_suite_identifier(value: &str) -> crate::Result<ViperTestSuiteIdentifier> {
    ViperTestSuiteIdentifier::try_from(value)
        .map_err(|error| error.to_string())
}

impl CreateViperSourceDescriptorCli {
    pub async fn execute(self, carl: &mut CarlClient, output: OutputFormat) -> crate::Result<()> {
        let descriptor = ViperSourceDescriptor {
            id: self.id.unwrap_or_else(ViperSourceId::random),
            name: self.name,
            url: self.url,
            kind: self.kind.into(),
        };

        create_viper_source_descriptor(descriptor, carl, &output).await
    }
}

pub async fn create_viper_source_descriptor(
    descriptor: ViperSourceDescriptor,
    carl: &mut CarlClient,
    output: &OutputFormat,
) -> crate::Result<()> {
    carl.viper.store_viper_source_descriptor(descriptor.clone()).await
        .map_err(|error| format!("Could not store VIPER source <{}>.\n  {error}", descriptor.id))?;

    let source = SerializableViperSource::from(descriptor);

    match output {
        OutputFormat::Table => {
            println!("Stored VIPER source '{}' with ID <{}>.", source.name, source.id);
        }
        OutputFormat::Json => {
            let json = serde_json::to_string(&source)
                .map_err(|error| format!("Could not serialize VIPER source.\n  {error}"))?;
            println!("{json}");
        }
        OutputFormat::PrettyJson => {
            let json = serde_json::to_string_pretty(&source)
                .map_err(|error| format!("Could not serialize VIPER source.\n  {error}"))?;
            println!("{json}");
        }
    }

    Ok(())
}
