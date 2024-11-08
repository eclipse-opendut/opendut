use serde::Deserialize;

use crate::specs::parse::ParseSpecificationError;
use crate::specs::parse::yaml::YamlSpecificationDocument;

#[derive(Debug, Deserialize)]
pub struct YamlSpecificationFile {
    pub documents: Vec<YamlSpecificationDocument>
}

impl YamlSpecificationFile {

    /// Tries to deserialize the given string of YAML text into a [YamlSpecificationFile].
    ///
    /// In contrast to [`YamlSpecificationDocument::try_from_yaml_str`] this function
    /// accepts a string containing one or more YAML documents.
    ///
    /// # Example
    ///
    /// ```
    /// # use opendut_types::specs::parse::yaml::YamlSpecificationFile;
    /// # fn main() -> Result<(), opendut_types::specs::parse::ParseSpecificationError> {
    /// let yaml = r#"
    /// ---
    /// kind: PeerDescriptor
    /// version: v1
    /// metadata:
    ///     id: 6d727331-8b2c-4300-abfa-45141d98c554
    ///     name: PeerA
    /// spec:
    ///     ...
    /// ---
    /// kind: PeerDescriptor
    /// version: v1
    /// metadata:
    ///     id: e541e3cd-c965-4dfe-9727-896b63be2bfb
    ///     name: PeerB
    /// spec:
    ///     ...
    /// "#;
    ///
    /// let file = YamlSpecificationFile::try_from_yaml_str(yaml)?;
    ///
    /// assert_eq!(file.documents.len(), 2);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_yaml_str(s: &str) -> Result<Self, ParseSpecificationError> {
        let documents = serde_yaml::Deserializer::from_str(s)
            .map(|yaml_document| {
                serde_yaml::Value::deserialize(yaml_document)
                    .and_then(serde_yaml::from_value::<YamlSpecificationDocument>)
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|cause| ParseSpecificationError::IllegalYamlSpecification { cause })?;
        Ok(Self { documents })
    }
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use googletest::prelude::*;
    use indoc::indoc;
    use uuid::uuid;
    use crate::specs::SpecificationMetadata;

    use super::*;

    #[test]
    pub fn test_that_a_string_containing_a_single_document_can_be_deserialized() -> Result<()> {

        let result = YamlSpecificationFile::try_from_yaml_str(indoc!("
            ---
            kind: AKnownKind
            version: SomeVersion
            metadata:
              id: 661f0c35-722e-4656-a7e7-6e976476b53b
              name: MyResource
            spec:
              xx: yy
        "));

        verify_that!(result, ok(anything()))?;

        let file = result?;

        verify_that!(file.documents, len(eq(1)))?;
        verify_that!(file.documents[0], matches_pattern!(YamlSpecificationDocument {
            kind: eq("AKnownKind"),
            version: eq("SomeVersion"),
            metadata: matches_pattern!(SpecificationMetadata {
                id: eq(&uuid!("661f0c35-722e-4656-a7e7-6e976476b53b")),
                name: eq("MyResource")
            })
        }))?;

        Ok(())
    }

    #[test]
    pub fn test_that_a_string_containing_multiple_documents_can_be_deserialized() -> Result<()> {
        let result = YamlSpecificationFile::try_from_yaml_str(indoc!("
            ---
            kind: AnEpicKind
            version: JustAGuessedVersion
            metadata:
              id: 00000000-0000-0000-0000-000000000000
              name: TheResourceNumberOne
            spec:
              a lot of bla bla
            ---
            kind: SomeKind
            version: AVersionIDontKnow
            metadata:
              id: e541e3cd-c965-4dfe-9727-896b63be2bfb
              name: MyEpicResource
            spec:
              ...
        "));

        verify_that!(result, ok(anything()))?;
        verify_that!(result?.documents, len(eq(2)))?;

        Ok(())
    }
}
