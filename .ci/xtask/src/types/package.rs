use clap::builder::PossibleValue;
use strum::IntoEnumIterator;

#[derive(Clone, Debug, strum::EnumIter)]
pub enum Package {
    Carl,
    CarlApi,
    Cleo,
    Edgar,
    Lea,
    NetbirdClientApi,
    Types,
    Util,
    Vpn,
    VpnNetbird,
    IntegrationTests,
}

impl Package {
    pub fn ident(&self) -> String {
        match self {
            Package::Carl => "opendut-carl",
            Package::CarlApi => "opendut-carl-api",
            Package::Cleo => "opendut-cleo",
            Package::Edgar => "opendut-edgar",
            Package::Lea => "opendut-lea",
            Package::NetbirdClientApi => "opendut-netbird-client-api",
            Package::Types => "opendut-types",
            Package::Util => "opendut-util",
            Package::Vpn => "opendut-vpn",
            Package::VpnNetbird => "opendut-vpn-netbird",
            Package::IntegrationTests => "opendut-integration-tests",
        }.to_string()
    }
}

impl std::fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ident())
    }
}


impl clap::ValueEnum for Package {
    fn value_variants<'a>() -> &'a [Package] {
        Box::leak(Self::iter().collect::<Vec<Package>>().into())
    }
    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(PossibleValue::new(self.ident()))
    }
}
