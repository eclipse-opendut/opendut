#[derive(Debug)]
pub enum MetadataError {
    WrongAttributeType {
        attribute: String,
        expected: String,
    },
    UnknownAttribute {
        attribute: String,
    },
}

impl MetadataError {

    pub(crate) fn new_wrong_attribute_type_error(
        attribute: impl Into<String>,
        expected: impl Into<String>,
    ) -> Self {
        Self::WrongAttributeType {
            attribute: attribute.into(),
            expected: expected.into(),
        }
    }

    pub(crate) fn new_unknown_attribute_error(
        attribute: impl Into<String>
    ) -> MetadataError {
        Self::UnknownAttribute {
            attribute: attribute.into(),
        }
    }
}
