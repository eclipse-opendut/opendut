#[derive(Debug, Clone, Copy)]
pub enum Crate {
    CargoDeny,
    CargoSbom,
    CargoBundleLicenses,
    Cross,
    Mdbook,
    MdbookPlantuml,
    Trunk,
}
impl Crate {
    pub fn ident(&self) -> String {
        match self {
            Crate::CargoDeny => "cargo-deny",
            Crate::CargoSbom => "cargo-sbom",
            Crate::CargoBundleLicenses => "cargo-bundle-licenses",
            Crate::Cross => "cross",
            Crate::Mdbook => "mdbook",
            Crate::MdbookPlantuml => "mdbook-plantuml",
            Crate::Trunk => "trunk",
        }.to_string()
    }
}
