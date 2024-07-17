use opendut_types::cluster::ClusterDeployment;
use crate::persistence::database::Db;
use super::{Persistable, PersistableConversionError};

#[derive(Debug)] //diesel::Queryable, diesel::Selectable, diesel::Insertable)]
// #[diesel(table_name = crate::persistence::database::schema::cluster_deployment)] //TODO create schema
// #[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistableClusterDeployment {
    //TODO
}
impl Persistable<ClusterDeployment> for PersistableClusterDeployment {
    fn insert(&self, db: Db) -> Option<Self> {
        todo!()
    }
}

impl From<ClusterDeployment> for PersistableClusterDeployment {
    fn from(value: ClusterDeployment) -> Self {
        todo!()
    }
}
impl TryFrom<PersistableClusterDeployment> for ClusterDeployment {
    type Error = PersistableConversionError<PersistableClusterDeployment, ClusterDeployment>;

    fn try_from(value: PersistableClusterDeployment) -> Result<Self, Self::Error> {
        todo!()
    }
}
