mod general;
mod parameters;
mod source;
mod peer;

pub use general::GeneralTab;
pub use parameters::ParametersTab;
pub use source::SourceTab;
pub use peer::PeerTab;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabIdentifier {
    #[default]
    General,
    ViperSource,
    Parameters,
    Peer,
}

impl TabIdentifier {
    const GENERAL_STR: &'static str = "general";
    const VIPER_SOURCE_STR: &'static str = "viper_source";
    const PARAMETERS_STR: &'static str = "parameters";
    const PEER_STR: &'static str = "peer";

    pub fn as_str(&self) -> &'static str {
        match self {
            TabIdentifier::General => TabIdentifier::GENERAL_STR,
            TabIdentifier::ViperSource => TabIdentifier::VIPER_SOURCE_STR,
            TabIdentifier::Parameters => TabIdentifier::PARAMETERS_STR,
            TabIdentifier::Peer => TabIdentifier::PEER_STR,
        }
    }
}

impl TryFrom<&str> for TabIdentifier {
    type Error = InvalidTabIdentifier;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            TabIdentifier::GENERAL_STR => Ok(TabIdentifier::General),
            TabIdentifier::VIPER_SOURCE_STR => Ok(TabIdentifier::ViperSource),
            TabIdentifier::PARAMETERS_STR => Ok(TabIdentifier::Parameters),
            TabIdentifier::PEER_STR => Ok(TabIdentifier::Peer),
            _ => Err(InvalidTabIdentifier { value: String::from(value) }),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Invalid tab identifier: {value}")]
pub struct InvalidTabIdentifier {
    value: String,
}
