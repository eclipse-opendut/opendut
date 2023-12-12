#[derive(Clone, Debug, clap::ValueEnum)]
pub enum Package {
    OpendutCarl,
    OpendutCarlApi,
    OpendutCleo,
    OpendutEdgar,
    OpendutLea,
    OpendutNetbirdClientApi,
    OpendutTypes,
    OpendutUtil,
    OpendutVpn,
    OpendutVpnNetbird,
    OpendutIntegrationTests,
}

impl Package {
    pub fn ident(&self) -> String {
        match self {
            Package::OpendutCarl => "opendut-carl",
            Package::OpendutCarlApi => "opendut-carl-api",
            Package::OpendutCleo => "opendut-cleo",
            Package::OpendutEdgar => "opendut-edgar",
            Package::OpendutLea => "opendut-lea",
            Package::OpendutNetbirdClientApi => "opendut-netbird-client-api",
            Package::OpendutTypes => "opendut-types",
            Package::OpendutUtil => "opendut-util",
            Package::OpendutVpn => "opendut-vpn",
            Package::OpendutVpnNetbird => "opendut-vpn-netbird",
            Package::OpendutIntegrationTests => "opendut-integration-tests",
        }.to_string()
    }
}

impl std::fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ident())
    }
}
