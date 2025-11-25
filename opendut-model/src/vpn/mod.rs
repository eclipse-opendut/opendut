use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VpnPeerConfiguration {
    Disabled,
    #[serde(rename_all = "kebab-case")]
    Netbird {
        management_url: Url,
        setup_key: netbird::SetupKey,
    }
}

pub mod netbird {
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct SetupKey { pub value: String }
    impl SetupKey {
        pub fn new(value: impl Into<String>) -> Self {
            Self { value: value.into() }
        }
    }

    impl From<Uuid> for SetupKey {
        fn from(value: Uuid) -> Self {
            Self { value: value.to_string() }
        }
    }
}
