use url::Url;
use crate::create_id_type;
use crate::viper::ViperTestSuiteIdentifier;

create_id_type!(ViperSourceId);


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViperSourceKind {
    Git,
    Http,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViperSourceDescriptor {
    pub id: ViperSourceId,
    pub name: ViperTestSuiteIdentifier,
    pub url: Url,
    pub kind: ViperSourceKind,
}
