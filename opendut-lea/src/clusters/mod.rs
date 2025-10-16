pub use card::ClustersCard;
pub use configurator::ClusterConfigurator;
pub use overview::ClustersOverview;
use serde::{Deserialize, Serialize};

mod card;
mod configurator;
mod overview;
mod components;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct IsDeployed(pub bool);
