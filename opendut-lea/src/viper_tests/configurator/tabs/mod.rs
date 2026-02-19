mod general;
mod cluster;
mod source;
mod suite;

pub use cluster::ClusterTab;
pub use general::GeneralTab;
pub use source::SourceTab;
pub use suite::SuiteTab;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabIdentifier {
    #[default]
    General,
    Source,
    Suite,
    Cluster,
}

impl TabIdentifier {
    const GENERAL_STR: &'static str = "general";
    const SOURCE_STR: &'static str = "source";
    const SUITE_STR: &'static str = "suite";
    const CLUSTER_STR: &'static str = "cluster";

    pub fn as_str(&self) -> &'static str {
        match self {
            TabIdentifier::General => TabIdentifier::GENERAL_STR,
            TabIdentifier::Source => TabIdentifier::SOURCE_STR,
            TabIdentifier::Suite => TabIdentifier::SUITE_STR,
            TabIdentifier::Cluster => TabIdentifier::CLUSTER_STR,
        }
    }
}

impl TryFrom<&str> for TabIdentifier {
    type Error = InvalidTabIdentifier;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            TabIdentifier::GENERAL_STR => Ok(TabIdentifier::General),
            TabIdentifier::SOURCE_STR => Ok(TabIdentifier::Source),
            TabIdentifier::SUITE_STR => Ok(TabIdentifier::Suite),
            TabIdentifier::CLUSTER_STR => Ok(TabIdentifier::Cluster),
            _ => Err(InvalidTabIdentifier { value: String::from(value) }),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Invalid tab identifier: {value}")]
pub struct InvalidTabIdentifier {
    value: String,
}
