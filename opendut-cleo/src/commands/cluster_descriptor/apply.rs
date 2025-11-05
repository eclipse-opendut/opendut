use std::collections::HashSet;
use opendut_model::cluster::{ClusterDescriptor, ClusterId, ClusterName};
use opendut_model::peer::PeerId;
use opendut_model::specs::cluster::ClusterDescriptorSpecificationV1;
use opendut_model::specs::SpecificationMetadata;
use opendut_model::topology::{DeviceId};

pub fn convert_document_to_cluster_descriptor(specification_metadata: SpecificationMetadata, cluster: ClusterDescriptorSpecificationV1) -> crate::Result<ClusterDescriptor>  {
    let SpecificationMetadata { id, name } = specification_metadata;
    
    let id = ClusterId::from(id);

    let name = ClusterName::try_from(name)
        .map_err(|error| error.to_string())?;
    
    let leader = cluster.leader_id;
    let devices = cluster.devices.into_iter()
        .map(DeviceId::from).
        collect::<HashSet<_>>();

    let configuration = ClusterDescriptor {
        id,
        name,
        leader: PeerId::from(leader),
        devices,
    };
    
    Ok(configuration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;
    
    #[test]
    fn should_convert_document_to_cluster_descriptor() -> anyhow::Result<()> {
        let cluster_descriptor = ClusterDescriptor {
            id: ClusterId::random(),
            name: ClusterName::try_from("FirstCluster")?,
            leader: PeerId::random(),
            devices: Default::default(),
        };
        
        let specification_meta_data = SpecificationMetadata {
            id: cluster_descriptor.id.uuid,
            name: cluster_descriptor.name.value().to_owned(),
        };
        
        let document = ClusterDescriptorSpecificationV1 {
            leader_id: cluster_descriptor.leader.uuid,
            devices: vec![],
        };
        
        let result = convert_document_to_cluster_descriptor(specification_meta_data, document).unwrap();

        assert_that!(result, eq(&cluster_descriptor));


        Ok(())      
    }
}
