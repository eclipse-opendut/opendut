use opendut_model::util::net::NetworkInterfaceName;
use rand::Rng;

pub trait NetworkInterfaceNameExt {
    fn with_random_suffix(base: &str) -> Self;
}

impl NetworkInterfaceNameExt for NetworkInterfaceName {
    fn with_random_suffix(base: &str) -> Self {
        let suffix: String = rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        NetworkInterfaceName::try_from(format!("{}-{}", base, suffix))
            .expect("Should be valid network interface")
    }
}
