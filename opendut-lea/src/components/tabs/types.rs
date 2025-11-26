use leptos::prelude::AnyView;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum TabIdentifier {
    #[default]
    General,
    Parameters,
    Devices,
    Network,
    Executor,
    Setup,
}

impl TabIdentifier {
    const GENERAL_STR: &'static str = "general";
    const PARAM_STR: &'static str = "parameters";
    const DEVICES_STR: &'static str = "devices";
    const NETWORK_STR: &'static str = "network";
    const EXECUTOR_STR: &'static str = "executor";
    const SETUP_STR: &'static str = "setup";

    pub fn as_str(&self) -> &'static str {
        match self {
            TabIdentifier::General => TabIdentifier::GENERAL_STR,
            TabIdentifier::Parameters => TabIdentifier::PARAM_STR,
            TabIdentifier::Devices => TabIdentifier::DEVICES_STR,
            TabIdentifier::Network => TabIdentifier::NETWORK_STR,
            TabIdentifier::Executor => TabIdentifier::EXECUTOR_STR,
            TabIdentifier::Setup => TabIdentifier::SETUP_STR,
        }
    }
}

impl TryFrom<&str> for TabIdentifier {
    type Error = InvalidTabIdentifier;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            TabIdentifier::GENERAL_STR => Ok(TabIdentifier::General),
            TabIdentifier::PARAM_STR => Ok(TabIdentifier::Parameters),
            TabIdentifier::DEVICES_STR => Ok(TabIdentifier::Devices),
            TabIdentifier::NETWORK_STR => Ok(TabIdentifier::Network),
            TabIdentifier::EXECUTOR_STR => Ok(TabIdentifier::Executor),
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

#[derive(Debug, Clone, Copy, Default)]
pub enum TabState {
    #[default]
    Normal,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: TabIdentifier,
    pub title: String,
    pub state: TabState,
    pub render: fn() -> AnyView,
}

impl Tab {
    pub fn new(id: TabIdentifier, title: String, state: TabState, render: fn() -> AnyView,) -> Self {
        Self { id, title, state, render }
    }
}
