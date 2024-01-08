#[derive(Debug, Clone, Copy)]
pub enum Crate {
    CargoDeny,
    CargoSbom,
    Cross,
    Mdbook,
    MdbookMermaid,
    Trunk,
}
impl Crate {
    pub fn ident(&self) -> String {
        match self {
            Crate::CargoDeny => "cargo-deny",
            Crate::CargoSbom => "cargo-sbom",
            Crate::Cross => "cross",
            Crate::Mdbook => "mdbook",
            Crate::MdbookMermaid => "mdbook-mermaid",
            Crate::Trunk => "trunk",
        }.to_string()
    }
}
