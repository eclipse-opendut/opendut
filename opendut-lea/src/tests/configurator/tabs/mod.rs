mod general;
mod parameter;

pub use general::GeneralTab;
pub use parameter::ParameterTab;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabIdentifier {
    #[default]
    General,
    Parameters,
}

impl TabIdentifier {
    const GENERAL_STR: &'static str = "general";
    const PARAMETERS_STR: &'static str = "parameters";

    pub fn as_str(&self) -> &'static str {
        match self {
            TabIdentifier::General => TabIdentifier::GENERAL_STR,
            TabIdentifier::Parameters => TabIdentifier::PARAMETERS_STR,
        }
    }
}

impl TryFrom<&str> for TabIdentifier {
    type Error = InvalidTabIdentifier;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            TabIdentifier::GENERAL_STR => Ok(TabIdentifier::General),
            TabIdentifier::PARAMETERS_STR => Ok(TabIdentifier::Parameters),
            _ => Err(InvalidTabIdentifier { value: String::from(value) }),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Invalid tab identifier: {value}")]
pub struct InvalidTabIdentifier {
    value: String,
}
