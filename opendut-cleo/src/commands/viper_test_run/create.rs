use std::collections::HashMap;

use opendut_carl_api::carl::CarlClient;
use opendut_model::peer::PeerId;
use opendut_model::viper::{ViperBindingValue, ViperParameterDescriptor, ViperParameterDescriptors, ViperParameterName, ViperSourceId, ViperTestId, ViperTestName, ViperTestParameters, ViperTestRunDescriptor};

use crate::OutputFormat;
use super::SerializableViperTestRun;

/// Create a VIPER test run descriptor
#[derive(clap::Parser)]
pub struct CreateViperTestRunDescriptorCli {
    /// Name of the test
    #[arg(short, long)]
    name: ViperTestName,
    /// ID of the source containing the test suite
    #[arg(long)]
    source_id: ViperSourceId,
    /// ID of the peer on which the test should run
    #[arg(long)]
    peer_id: PeerId,
    /// Parameter binding as NAME=VALUE; repeat for multiple parameters. Omitted parameters use suite defaults.
    #[arg(short = 'p', long = "parameter", value_parser = parse_parameter)]
    parameters: Vec<(ViperParameterName, String)>,
    /// ID of the test run; generates a random UUID when omitted
    #[arg(short, long)]
    id: Option<ViperTestId>,
}

impl CreateViperTestRunDescriptorCli {
    pub async fn execute(self, carl: &mut CarlClient, output: OutputFormat) -> crate::Result<()> {
        let peer_id = self.peer_id;
        carl.peers.get_peer_descriptor(peer_id).await
            .map_err(|error| format!("Failed to retrieve peer descriptor <{peer_id}>.\n  {error}"))?;

        let source_id = self.source_id;
        let suite = carl.viper.get_viper_test_suite_parameters(source_id).await
            .map_err(|error| format!("Failed to retrieve VIPER test suite parameters for source <{source_id}>.\n  {error}"))?;

        let descriptor = ViperTestRunDescriptor {
            id: self.id.unwrap_or_else(ViperTestId::random),
            name: self.name,
            source: source_id,
            peer: peer_id,
            parameters: bind_parameters(suite.parameters, self.parameters)?,
        };

        create_viper_test_run_descriptor(descriptor, carl, &output).await
    }
}

pub async fn create_viper_test_run_descriptor(
    descriptor: ViperTestRunDescriptor,
    carl: &mut CarlClient,
    output: &OutputFormat,
) -> crate::Result<()> {
    carl.viper.store_viper_test_run_descriptor(descriptor.clone()).await
        .map_err(|error| format!("Could not store VIPER test run descriptor <{}>.\n  {error}", descriptor.id))?;

    let test = SerializableViperTestRun::from(descriptor);

    match output {
        OutputFormat::Table => {
            println!("Stored VIPER test '{}' with ID <{}>.", test.name, test.id);
        }
        OutputFormat::Json => {
            let json = serde_json::to_string(&test)
                .map_err(|error| format!("Could not serialize VIPER test run descriptor.\n  {error}"))?;
            println!("{json}");
        }
        OutputFormat::PrettyJson => {
            let json = serde_json::to_string_pretty(&test)
                .map_err(|error| format!("Could not serialize VIPER test run descriptor.\n  {error}"))?;
            println!("{json}");
        }
    }

    Ok(())
}

fn parse_parameter(value: &str) -> crate::Result<(ViperParameterName, String)> {
    let (name, value) = value.split_once('=')
        .ok_or_else(|| String::from("Expected a parameter binding in the form NAME=VALUE."))?;
    let name = ViperParameterName::try_from(name)
        .map_err(|error| error.to_string())?;

    Ok((name, value.to_owned()))
}

fn bind_parameters(
    descriptors: ViperParameterDescriptors,
    parameters: Vec<(ViperParameterName, String)>,
) -> crate::Result<ViperTestParameters> {
    let mut inputs = HashMap::new();
    for (name, value) in parameters {
        if inputs.insert(name.clone(), value).is_some() {
            return Err(format!("VIPER test parameter '{name}' was specified multiple times."));
        }
    }

    for name in inputs.keys() {
        if !descriptors.iter().any(|descriptor| descriptor.name() == name) {
            return Err(format!("Unknown VIPER test parameter '{name}'."));
        }
    }

    descriptors.into_iter()
        .map(|descriptor| {
            let name = descriptor.name().clone();
            let value = match inputs.remove(&name) {
                Some(value) => Some(parse_binding_value(&descriptor, &value)
                    .map_err(|error| format!("Invalid value for VIPER test parameter '{name}'.\n  {error}"))?),
                None if descriptor.has_default_value() => None,
                None => return Err(format!("Missing required VIPER test parameter '{name}'. Specify --parameter {name}=VALUE.")),
            };

            Ok((name, value))
        })
        .collect()
}

fn parse_binding_value(descriptor: &ViperParameterDescriptor, value: &str) -> crate::Result<ViperBindingValue> {
    match descriptor {
        ViperParameterDescriptor::BooleanParameter { .. } => {
            value.parse::<bool>()
                .map(ViperBindingValue::BooleanValue)
                .map_err(|_| String::from("Expected 'true' or 'false'."))
        }
        ViperParameterDescriptor::NumberParameter { .. } => {
            let value = value.parse::<i64>()
                .map_err(|error| format!("Expected a 64-bit integer.\n  {error}"))?;
            descriptor.validate_number_parameter(value)
                .map_err(|error| error.to_string())?;
            Ok(ViperBindingValue::NumberValue(value))
        }
        ViperParameterDescriptor::TextParameter { .. } => {
            descriptor.validate_text_parameter(value)
                .map_err(|error| error.to_string())?;
            Ok(ViperBindingValue::TextValue(value.to_owned()))
        }
    }
}
