mod general;

pub use general::GeneralTab;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabIdentifier {
    #[default]
    General,
}

impl TabIdentifier {
    const GENERAL_STR: &'static str = "general";

    pub fn as_str(&self) -> &'static str {
        match self {
            TabIdentifier::General => TabIdentifier::GENERAL_STR,
        }
    }
}

impl TryFrom<&str> for TabIdentifier {
    type Error = InvalidTabIdentifier;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            TabIdentifier::GENERAL_STR => Ok(TabIdentifier::General),
            _ => Err(InvalidTabIdentifier {
                value: String::from(value),
            }),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Invalid tab identifier: {value}")]
pub struct InvalidTabIdentifier {
    value: String,
}
