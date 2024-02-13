pub use devices::DevicesTab;
pub use general::GeneralTab;
pub use setup::SetupTab;

mod devices;
mod general;
mod setup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabIdentifier {
    #[default]
    General,
    Devices,
    Setup,
}

impl TabIdentifier {
    const GENERAL_STR: &'static str = "general";
    const DEVICES_STR: &'static str = "devices";
    const SETUP_STR: &'static str = "setup";

    pub fn to_str(&self) -> &'static str {
        match self {
            TabIdentifier::General => TabIdentifier::GENERAL_STR,
            TabIdentifier::Devices => TabIdentifier::DEVICES_STR,
            TabIdentifier::Setup => TabIdentifier::SETUP_STR,
        }
    }
}

impl TryFrom<&str> for TabIdentifier {
    type Error = InvalidTabIdentifier;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            TabIdentifier::GENERAL_STR => Ok(TabIdentifier::General),
            TabIdentifier::DEVICES_STR => Ok(TabIdentifier::Devices),
            TabIdentifier::SETUP_STR => Ok(TabIdentifier::Setup),
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
