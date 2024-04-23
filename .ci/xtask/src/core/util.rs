use std::fs;
use std::process::Command;
use anyhow::anyhow;

use assert_fs::assert::PathAssert;
use tracing_subscriber::fmt::format::FmtSpan;

use crate::core::dependency::Crate;
use crate::core::metadata;
use crate::core::types::Arch;

#[tracing::instrument(level = tracing::Level::TRACE)]
pub fn install_crate(install: Crate) -> crate::Result {
    let cargo_metadata = metadata::cargo();

    let version = cargo_metadata.workspace_metadata["ci"]["xtask"][install.ident()]["version"].as_str()
        .unwrap_or_else(|| panic!("No version information for crate '{}' in root Cargo.toml. Aborting installation.", install.ident()));

    Command::new("cargo")
        .arg("install")
        .arg("--version").arg(version)
        .arg(install.ident())
        .run_requiring_success()
}

#[tracing::instrument]
pub fn install_toolchain(arch: Arch) -> crate::Result {
    Command::new("rustup")
        .args(["target", "add", &arch.triple()])
        .run_requiring_success()
}


pub trait RunRequiringSuccess {
    fn run_requiring_success(&mut self) -> crate::Result;
}
impl RunRequiringSuccess for Command {
    fn run_requiring_success(&mut self) -> crate::Result {
        let status = self.status()
            .expect("Error while running command");

        if status.success() {
            Ok(())
        } else {
            let mut error = format!("Error while running command: {self:?}\n");
            if let Some(status) = &status.code() {
                error += format!("  Exited with status code {}.\n", status).as_ref();
            }
            Err(anyhow!(error))
        }
    }
}


pub fn init_tracing() -> crate::Result {
    use tracing_subscriber::filter::{EnvFilter, LevelFilter};

    let tracing_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env()?
        .add_directive("opendut=trace".parse()?);

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_env_filter(tracing_filter)
        .compact()
        .init();
    Ok(())
}


pub mod file {
    use std::os::unix::fs::MetadataExt;

    use assert_fs::fixture::ChildPath;

    use super::*;

    pub trait ChildPathExt {
        fn assert_non_empty_file(&self);
        fn file_name_str(&self) -> &str;
        fn dir_contains_exactly_in_order(&self, paths: Vec<&ChildPath>);
    }
    impl ChildPathExt for ChildPath {
        fn assert_non_empty_file(&self) {
            self.assert(predicates::path::is_file());

            assert!(
                self.metadata().unwrap().size() > 0,
                "{:?} is empty", self.path()
            );
        }

        fn file_name_str(&self) -> &str {
            self.file_name().unwrap()
                .to_str().unwrap()
        }

        fn dir_contains_exactly_in_order(&self, paths: Vec<&ChildPath>) {
            let mut sub_paths = fs::read_dir(self).unwrap()
                .map(|entry| entry.unwrap().path())
                .collect::<Vec<_>>();

            sub_paths.sort();

            let mut sub_paths = sub_paths.into_iter();

            for entry in paths {
                let actual = sub_paths.next();
                let expected = Some(entry.to_path_buf());
                assert_eq!(
                    actual, expected.clone(),
                    "Found '{:?}' as next path in alphabetical order, but expected '{:?}'.", actual, expected
                );
            }

            let actual = sub_paths.next();
            assert_eq!(
                actual, None,
                "Found path '{:?}' in directory, but expected no further paths.", actual
            );
        }
    }
}
