use crate::cluster::ClusterId;
use crate::viper::ViperRunId;


#[derive(Clone, Debug)]
pub struct ViperRunDeployment {
    pub id: ViperRunId,
    pub cluster: ClusterId,
}
