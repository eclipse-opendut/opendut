use cicero::commands::{Cli, Crate};

pub static CARGO_BUNDLE_LICENSES: Cli = Crate::new("cargo-bundle-licenses").into_cli();

pub static CARGO_DENY: Cli = Crate::new("cargo-deny").into_cli();

pub static CARGO_SBOM: Cli = Crate::new("cargo-sbom").into_cli();

pub static CARGO_TARPAULIN: Cli = Crate::new("cargo-tarpaulin").into_cli();

pub static CROSS: Cli = Crate::new("cross")
    .into_cli()
    .with_base_command(&|mut command| {
        command
            .arg("build");
        command
    });

pub static MDBOOK: Cli = Crate::new("mdbook").into_cli()
    .with_crate_dependencies(&[
        Crate::new("mdbook-alerts"),
        Crate::new("mdbook-plantuml"),
    ]);

pub static TRUNK: Cli = Crate::new("trunk").with_install_args(&["--locked"]).into_cli();
