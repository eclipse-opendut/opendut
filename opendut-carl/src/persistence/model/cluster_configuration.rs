use opendut_types::cluster::ClusterConfiguration;

use super::{Persistable, PersistableConversionError};

// #[derive(diesel::Queryable, diesel::Selectable, diesel::Insertable)]
// #[diesel(table_name = crate::persistence::database::schema::cluster_configuration)] //TODO create schema
// #[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistableClusterConfiguration {
    //TODO
}
impl Persistable<ClusterConfiguration> for PersistableClusterConfiguration {}

impl From<ClusterConfiguration> for PersistableClusterConfiguration {
    fn from(value: ClusterConfiguration) -> Self {
        todo!()
    }
}
impl TryFrom<PersistableClusterConfiguration> for ClusterConfiguration {
    type Error = PersistableConversionError<PersistableClusterConfiguration, ClusterConfiguration>;

    fn try_from(value: PersistableClusterConfiguration) -> Result<Self, Self::Error> {
        todo!()
    }
}
