pub use devices::DevicesTab;
pub use general::GeneralTab;
pub use leader::LeaderTab;

mod general;
mod devices;
mod leader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabIdentifier {
    General,
    Devices,
    Leader,
}

impl TabIdentifier {
    const GENERAL_STR: &'static str = "general";
    const DEVICES_STR: &'static str = "devices";

    const LEADER_STR: &'static str = "leader";

    pub fn to_str(&self) -> &'static str {
        match self {
            TabIdentifier::General => TabIdentifier::GENERAL_STR,
            TabIdentifier::Devices => TabIdentifier::DEVICES_STR,
            TabIdentifier::Leader => TabIdentifier::LEADER_STR,
        }
    }
}

impl Default for TabIdentifier {
    fn default() -> Self {
        TabIdentifier::General
    }
}

impl TryFrom<&str> for TabIdentifier {
    type Error = InvalidTabIdentifier;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            TabIdentifier::GENERAL_STR => Ok(TabIdentifier::General),
            TabIdentifier::DEVICES_STR => Ok(TabIdentifier::Devices),
            TabIdentifier::LEADER_STR => Ok(TabIdentifier::Leader),
            _ => Err(InvalidTabIdentifier { value: String::from(value) }),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Invalid tab identifier: {value}")]
pub struct InvalidTabIdentifier {
    value: String,
}
