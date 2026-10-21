use opendut_model::peer::PeerId;
use opendut_model::specs::SpecificationMetadata;
use opendut_model::specs::viper_test_run::{ViperParameterValueSpecificationV1, ViperTestRunDescriptorSpecificationV1};
use opendut_model::viper::{ViperBindingValue, ViperParameterName, ViperSourceId, ViperTestId, ViperTestName, ViperTestParameters, ViperTestRunDescriptor};

pub fn convert_document_to_viper_test_run_descriptor(specification_metadata: SpecificationMetadata, viper_test_run: ViperTestRunDescriptorSpecificationV1) -> crate::Result<ViperTestRunDescriptor> {
    let SpecificationMetadata { id, name } = specification_metadata;

    let id = ViperTestId::from(id);
    let name = ViperTestName::try_from(name)
        .map_err(|error| format!("Failed to parse VIPER test run name: {error}"))?;

    let parameters = viper_test_run.parameters.into_iter()
        .map(|(name, value)| {
            let name = ViperParameterName::try_from(name)
                .map_err(|error| format!("Failed to parse VIPER test parameter name: {error}"))?;

            let value = match value {
                ViperParameterValueSpecificationV1::Boolean(value) => ViperBindingValue::BooleanValue(value),
                ViperParameterValueSpecificationV1::Number(value) => ViperBindingValue::NumberValue(value),
                ViperParameterValueSpecificationV1::Text(value) => ViperBindingValue::TextValue(value),
            };

            Ok((name, Some(value)))
        })
        .collect::<crate::Result<ViperTestParameters>>()?;

    Ok(ViperTestRunDescriptor {
        id,
        name,
        source: ViperSourceId::from(viper_test_run.source_id),
        peer: PeerId::from(viper_test_run.peer_id),
        parameters,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;
    use std::collections::HashMap;

    #[test]
    fn should_convert_document_to_viper_test_run_descriptor() -> anyhow::Result<()> {
        let viper_test_run_descriptor = ViperTestRunDescriptor {
            id: ViperTestId::random(),
            name: ViperTestName::try_from("FirstTestRun")?,
            source: ViperSourceId::random(),
            peer: PeerId::random(),
            parameters: vec![],
        };

        let specification_metadata = SpecificationMetadata {
            id: viper_test_run_descriptor.id.uuid,
            name: viper_test_run_descriptor.name.value().to_owned(),
        };

        let document = ViperTestRunDescriptorSpecificationV1 {
            source_id: viper_test_run_descriptor.source.uuid,
            peer_id: viper_test_run_descriptor.peer.uuid,
            parameters: HashMap::new(),
        };

        let result = convert_document_to_viper_test_run_descriptor(specification_metadata, document).unwrap();

        assert_that!(result, eq(&viper_test_run_descriptor));

        Ok(())
    }

    #[test]
    fn should_convert_viper_test_run_parameters() -> anyhow::Result<()> {
        let specification_metadata = SpecificationMetadata {
            id: ViperTestId::random().uuid,
            name: String::from("FirstTestRun"),
        };

        let document = ViperTestRunDescriptorSpecificationV1 {
            source_id: ViperSourceId::random().uuid,
            peer_id: PeerId::random().uuid,
            parameters: HashMap::from([
                (String::from("enabled"), ViperParameterValueSpecificationV1::Boolean(false)),
                (String::from("count"), ViperParameterValueSpecificationV1::Number(-42)),
                (String::from("message"), ViperParameterValueSpecificationV1::Text(String::from("true"))),
            ]),
        };

        let result = convert_document_to_viper_test_run_descriptor(specification_metadata, document).unwrap();

        assert_that!(result.parameters, unordered_elements_are![
            eq(&(ViperParameterName::try_from("enabled")?, Some(ViperBindingValue::BooleanValue(false)))),
            eq(&(ViperParameterName::try_from("count")?, Some(ViperBindingValue::NumberValue(-42)))),
            eq(&(ViperParameterName::try_from("message")?, Some(ViperBindingValue::TextValue(String::from("true"))))),
        ]);

        Ok(())
    }

    #[test]
    fn should_reject_invalid_viper_test_run_name() {
        for name in [String::new(), String::from("abc"), "a".repeat(65), String::from("Invalid?Name")] {
            let specification_metadata = SpecificationMetadata {
                id: ViperTestId::random().uuid,
                name,
            };

            let document = ViperTestRunDescriptorSpecificationV1 {
                source_id: ViperSourceId::random().uuid,
                peer_id: PeerId::random().uuid,
                parameters: HashMap::new(),
            };

            let result = convert_document_to_viper_test_run_descriptor(specification_metadata, document);

            assert_that!(result, err(starts_with("Failed to parse VIPER test run name: ")));
        }
    }

    #[test]
    fn should_reject_invalid_viper_test_parameter_name() {
        for name in ["", "Invalid Parameter"] {
            let specification_metadata = SpecificationMetadata {
                id: ViperTestId::random().uuid,
                name: String::from("FirstTestRun"),
            };

            let document = ViperTestRunDescriptorSpecificationV1 {
                source_id: ViperSourceId::random().uuid,
                peer_id: PeerId::random().uuid,
                parameters: HashMap::from([
                    (name.to_owned(), ViperParameterValueSpecificationV1::Boolean(true)),
                ]),
            };

            let result = convert_document_to_viper_test_run_descriptor(specification_metadata, document);

            assert_that!(result, err(starts_with("Failed to parse VIPER test parameter name: ")));
        }
    }
}
