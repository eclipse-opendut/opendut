use opendut_types::cluster::*;

use super::*;

#[derive(Clone)]
pub struct ParseableClusterId(pub ClusterId);
impl FromStr for ParseableClusterId {
    type Err = ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let inner = ClusterId::try_from(value)
            .map_err(|cause| ParseError::new::<Self>(value, cause.to_string()))?;
        Ok(Self(inner))
    }
}

#[derive(Clone)]
pub struct ParseableClusterName(pub ClusterName);
impl FromStr for ParseableClusterName {
    type Err = ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let inner = ClusterName::try_from(value)
            .map_err(|cause| ParseError::new::<Self>(value, cause.to_string()))?;
        Ok(Self(inner))
    }
}
