use crate::fs;
use std::process::Command;
use anyhow::anyhow;

use assert_fs::assert::PathAssert;

use crate::core::types::Arch;

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
            .unwrap_or_else(|cause| panic!("Error while running command: {self:?}\n  {cause}"));

        if status.success() {
            Ok(())
        } else {
            let mut error = format!("Error while running command: {self:?}\n");
            if let Some(status) = &status.code() {
                error += format!("  Exited with status code {status}.\n").as_ref();
            }
            Err(anyhow!(error))
        }
    }
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
            let mut sub_paths = fs::read_dir(self.to_path_buf()).unwrap()
                .map(|entry| entry.unwrap().path())
                .collect::<Vec<_>>();

            sub_paths.sort();

            let mut sub_paths = sub_paths.into_iter();

            for entry in paths {
                let actual = sub_paths.next();
                let expected = Some(entry.to_path_buf());
                assert_eq!(
                    actual, expected.clone(),
                    "Found '{actual:?}' as next path in alphabetical order, but expected '{expected:?}'."
                );
            }

            let actual = sub_paths.next();
            assert_eq!(
                actual, None,
                "Found path '{actual:?}' in directory, but expected no further paths."
            );
        }
    }
}
