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

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct SetupKey { pub value: String }

    impl From<String> for SetupKey {
        fn from(value: String) -> Self {
            Self { value }
        }
    }

    impl From<&str> for SetupKey {
        fn from(value: &str) -> Self {
            Self { value: value.into() }
        }
    }
}
