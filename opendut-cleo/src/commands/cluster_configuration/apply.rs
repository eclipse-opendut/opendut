use opendut_types::cluster::{ClusterConfiguration, ClusterId, ClusterName};
use opendut_types::peer::PeerId;
use opendut_types::specs::cluster::ClusterConfigurationSpecificationV1;
use opendut_types::specs::SpecificationMetadata;

pub fn convert_document_to_cluster_configuration(specification_metadata: SpecificationMetadata, cluster: ClusterConfigurationSpecificationV1) -> crate::Result<ClusterConfiguration>  {
    let SpecificationMetadata { id, name } = specification_metadata;
    
    let id = ClusterId::from(id);

    let name = ClusterName::try_from(name)
        .map_err(|error| error.to_string())?;
    
    let leader = cluster.leader_id; 
    
    let configuration = ClusterConfiguration {
        id,
        name,
        leader: PeerId::from(leader),
        devices: Default::default(), // TODO
    };
    
    Ok(configuration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;
    
    #[test]
    fn should_convert_document_to_cluster_configuration() -> anyhow::Result<()> {
        let cluster_configuration = ClusterConfiguration {
            id: ClusterId::random(),
            name: ClusterName::try_from("FirstCluster")?,
            leader: PeerId::random(),
            devices: Default::default(),
        };
        
        let specification_meta_data = SpecificationMetadata {
            id: cluster_configuration.id.clone().0,
            name: cluster_configuration.name.clone().value(),
        };
        
        let document = ClusterConfigurationSpecificationV1 {
            leader_id: cluster_configuration.leader.uuid,
        };
        
        let result = convert_document_to_cluster_configuration(specification_meta_data, document).unwrap();

        assert_that!(result, eq(&cluster_configuration));


        Ok(())      
    }
}