use std::error::Error;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use opendut_model::cluster::ClusterId;
use opendut_model::peer::PeerId;

#[derive(thiserror::Error, Debug)]
#[error("Cannot create GroupName from '{value}':\n  {cause}")]
pub struct InvalidGroupNameError {
    value: String,
    cause: Box<dyn Error>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum GroupName {
    Peer(PeerId),
    Cluster(ClusterId),
    Other(String),
}

impl GroupName {
    const PEER_GROUP_PREFIX: &'static str = "opendut-peer-group-";
    const CLUSTER_GROUP_PREFIX: &'static str = "opendut-cluster-group-";
}

impl From<PeerId> for GroupName {
    fn from(value: PeerId) -> Self {
        Self::Peer(value)
    }
}

impl From<ClusterId> for GroupName {
    fn from(value: ClusterId) -> Self {
        Self::Cluster(value)
    }
}

impl TryFrom<&str> for GroupName {

    type Error = InvalidGroupNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some(uuid) = value.strip_prefix(GroupName::PEER_GROUP_PREFIX) {
            PeerId::try_from(uuid)
                .map(Self::Peer)
                .map_err(|cause| InvalidGroupNameError { value: value.to_owned(), cause: cause.into() })
        }
        else if let Some(uuid) = value.strip_prefix(GroupName::CLUSTER_GROUP_PREFIX) {
            ClusterId::try_from(uuid)
                .map(Self::Cluster)
                .map_err(|cause| InvalidGroupNameError { value: value.to_owned(), cause: cause.into() })
        }
        else {
            Ok(Self::Other(value.to_owned()))
        }
    }
}

impl TryFrom<String> for GroupName {

    type Error = InvalidGroupNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        GroupName::try_from(value.as_str())
    }
}

impl From<&GroupName> for String {
    fn from(value: &GroupName) -> Self {
        match value {
            GroupName::Peer(id) => format!("{}{}", GroupName::PEER_GROUP_PREFIX, id.uuid),
            GroupName::Cluster(id) => format!("{}{}", GroupName::CLUSTER_GROUP_PREFIX, id.0),
            GroupName::Other(name) => name.to_owned(),
        }
    }
}

impl From<GroupName> for String {
    fn from(value: GroupName) -> Self {
        String::from(&value)
    }
}

impl Display for GroupName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use googletest::prelude::*;

    use super::*;

    #[test]
    fn A_GroupName_for_a_peer_should_be_convertable_from_and_to_string() -> anyhow::Result<()> {

        let uuid = "4f8aca21-512d-494d-8b34-5eb7510d42f1";
        let peer_id = PeerId::try_from(uuid)?;
        let group_name = "opendut-peer-group-4f8aca21-512d-494d-8b34-5eb7510d42f1";

        let from_string = GroupName::try_from(String::from(group_name));
        let from_id = GroupName::from(peer_id);

        assert_that!(from_string.as_ref(), ok(eq(&GroupName::Peer(peer_id))));
        assert_that!(&from_id, eq(&GroupName::Peer(peer_id)));
        assert_that!(String::from(from_string.unwrap()), eq(group_name));
        assert_that!(String::from(from_id), eq(group_name));

        Ok(())
    }

    #[test]
    fn A_GroupName_for_a_peer_should_not_be_convertable_from_string_containing_an_invalid_uuid() {

        let group_name = "opendut-peer-group-this-is-an-invalid-uuid";

        assert_that!(GroupName::try_from(String::from(group_name)), err(anything()));
    }

    #[test]
    fn A_GroupName_for_a_cluster_should_be_convertable_from_and_to_string() -> anyhow::Result<()> {

        let uuid = "5c806e1c-448e-4dda-854a-20a33cfe1cfe";
        let cluster_id = ClusterId::try_from(uuid)?;
        let group_name = "opendut-cluster-group-5c806e1c-448e-4dda-854a-20a33cfe1cfe";

        let from_string = GroupName::try_from(String::from(group_name));
        let from_id = GroupName::from(cluster_id);

        assert_that!(from_string.as_ref(), ok(eq(&GroupName::Cluster(cluster_id))));
        assert_that!(&from_id, eq(&GroupName::Cluster(cluster_id)));
        assert_that!(String::from(from_string.unwrap()), eq(group_name));
        assert_that!(String::from(from_id), eq(group_name));

        Ok(())
    }

    #[test]
    fn A_GroupName_for_a_cluster_should_not_be_convertable_from_string_containing_an_invalid_uuid() {

        let group_name = "opendut-cluster-group-this-is-an-invalid-uuid";

        assert_that!(GroupName::try_from(String::from(group_name)), err(anything()));
    }

    #[test]
    fn A_GroupName_for_anything_else_should_be_convertable_from_and_to_string() {

        let group_name = "a-cool-group";

        let from_string = GroupName::try_from(String::from(group_name));

        assert_that!(from_string.as_ref(), ok(eq(&GroupName::Other(String::from(group_name)))));
        assert_that!(String::from(from_string.unwrap()), eq(group_name));
    }
}
