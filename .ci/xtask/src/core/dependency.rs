#[derive(Debug, Clone, Copy)]
pub enum Crate {
    CargoDeny,
    CargoBundleLicenses,
    CargoSbom,
    CargoTarpaulin,
    Cross,
    Mdbook,
    MdbookPlantuml,
    Trunk,
}
impl Crate {
    pub fn ident(&self) -> String {
        match self {
            Crate::CargoDeny => "cargo-deny",
            Crate::CargoBundleLicenses => "cargo-bundle-licenses",
            Crate::CargoSbom => "cargo-sbom",
            Crate::Cross => "cross",
            Crate::Mdbook => "mdbook",
            Crate::MdbookPlantuml => "mdbook-plantuml",
            Crate::CargoTarpaulin => "cargo-tarpaulin",
            Crate::Trunk => "trunk",
        }.to_string()
    }
}
