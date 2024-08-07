#[derive(Debug, Clone, Copy)]
pub enum Crate {
    CargoDeny,
    CargoBundleLicenses,
    CargoSbom,
    CargoTarpaulin,
    Cross,
    DieselCli,
    Mdbook,
    MdbookPlantuml,
    Trunk,
}
impl Crate {
    pub fn ident(&self) -> &'static str {
        match self {
            Crate::CargoDeny => "cargo-deny",
            Crate::CargoBundleLicenses => "cargo-bundle-licenses",
            Crate::CargoSbom => "cargo-sbom",
            Crate::Cross => "cross",
            Crate::DieselCli => "diesel_cli",
            Crate::Mdbook => "mdbook",
            Crate::MdbookPlantuml => "mdbook-plantuml",
            Crate::CargoTarpaulin => "cargo-tarpaulin",
            Crate::Trunk => "trunk",
        }
    }
    pub fn install_command_args(&self) -> &[&'static str] {
        match self {
            Crate::DieselCli => &["--no-default-features", "--features", "postgres-bundled"],
            Crate::Trunk => &["--locked"],
            _ => &[],
        }
    }
}
