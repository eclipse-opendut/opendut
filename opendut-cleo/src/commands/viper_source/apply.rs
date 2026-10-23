use opendut_model::specs::SpecificationMetadata;
use opendut_model::specs::viper_source::{ViperSourceDescriptorSpecificationV1, ViperSourceKindSpecificationV1};
use opendut_model::viper::{ViperSourceDescriptor, ViperSourceId, ViperSourceKind, ViperTestSuiteIdentifier};

pub fn convert_document_to_viper_source_descriptor(specification_metadata: SpecificationMetadata, viper_source: ViperSourceDescriptorSpecificationV1) -> crate::Result<ViperSourceDescriptor> {
    let SpecificationMetadata { id, name } = specification_metadata;

    let id = ViperSourceId::from(id);
    let name = ViperTestSuiteIdentifier::try_from(name)
        .map_err(|error| format!("Failed to parse viper source name: {:?}", error))?;

    let kind = match viper_source.kind {
        ViperSourceKindSpecificationV1::Git => ViperSourceKind::Git,
        ViperSourceKindSpecificationV1::Http => ViperSourceKind::Http,
    };

    Ok(ViperSourceDescriptor {
        id,
        name,
        url: viper_source.url,
        kind,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;
    use url::Url;

    #[test]
    fn should_convert_document_to_git_viper_source_descriptor() -> anyhow::Result<()> {
        let viper_source_descriptor = ViperSourceDescriptor {
            id: ViperSourceId::random(),
            name: ViperTestSuiteIdentifier::try_from("FirstTestSuite")?,
            url: Url::parse("https://example.com/test-suite.git")?,
            kind: ViperSourceKind::Git,
        };

        let specification_metadata = SpecificationMetadata {
            id: viper_source_descriptor.id.uuid,
            name: viper_source_descriptor.name.to_string(),
        };

        let document = ViperSourceDescriptorSpecificationV1 {
            url: viper_source_descriptor.url.clone(),
            kind: ViperSourceKindSpecificationV1::Git,
        };

        let result = convert_document_to_viper_source_descriptor(specification_metadata, document).unwrap();

        assert_that!(result, eq(&viper_source_descriptor));

        Ok(())
    }

    #[test]
    fn should_convert_document_to_http_viper_source_descriptor() -> anyhow::Result<()> {
        let viper_source_descriptor = ViperSourceDescriptor {
            id: ViperSourceId::random(),
            name: ViperTestSuiteIdentifier::try_from("FirstTestSuite")?,
            url: Url::parse("https://example.com/test-suite.tar.gz")?,
            kind: ViperSourceKind::Http,
        };

        let specification_metadata = SpecificationMetadata {
            id: viper_source_descriptor.id.uuid,
            name: viper_source_descriptor.name.to_string(),
        };

        let document = ViperSourceDescriptorSpecificationV1 {
            url: viper_source_descriptor.url.clone(),
            kind: ViperSourceKindSpecificationV1::Http,
        };

        let result = convert_document_to_viper_source_descriptor(specification_metadata, document).unwrap();

        assert_that!(result, eq(&viper_source_descriptor));

        Ok(())
    }

    #[test]
    fn should_reject_invalid_viper_source_name() -> anyhow::Result<()> {
        for name in ["", "Invalid Test Suite"] {
            let specification_metadata = SpecificationMetadata {
                id: ViperSourceId::random().uuid,
                name: name.to_owned(),
            };

            let document = ViperSourceDescriptorSpecificationV1 {
                url: Url::parse("https://example.com/test-suite.git")?,
                kind: ViperSourceKindSpecificationV1::Git,
            };

            let result = convert_document_to_viper_source_descriptor(specification_metadata, document);

            assert!(result.is_err());
        }

        Ok(())
    }
}
