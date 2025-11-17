use crate::core::util::RunRequiringSuccess;
use anyhow::anyhow;
use cicero::path::repo_path;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tracing::debug;
use crate::core::commands::CROSS;
use crate::core::types::Arch;

/// Run integration tests of EDGAR
#[derive(Debug, clap::Parser)]
pub struct IntegrationTestCli {
}

impl IntegrationTestCli {
    pub fn default_handling(self) -> crate::Result { test(self) }
}

#[derive(Debug, Deserialize, Clone)]
struct CargoCompilerTarget {
    name: String,
}

#[derive(Debug, Deserialize)]
struct CargoCompilerOutput {
    reason: String,
    executable: Option<String>,
    target: Option<CargoCompilerTarget>,
}

impl CargoCompilerOutput {
    fn edgar_test_binary(&self) -> Option<PathBuf> {
        match (self.reason.as_ref(), self.executable.as_ref(), self.target.clone()) {
            ("compiler-artifact", Some(executable), Some(target)) => {
                if target.name == "opendut_edgar" {
                    Some(PathBuf::from(executable))
                } else {
                    None
                }
            },
            (_, _, _) => None
        }
    }
}


#[tracing::instrument(skip_all)]
pub fn test(_params: IntegrationTestCli) -> crate::Result {
    let edgar_test_binary = create_edgar_integration_test_binary()?;
    debug!("EDGAR test binary: {}", edgar_test_binary);
    
    run_edgar_integration_test_binary_in_docker(edgar_test_binary)?;

    Ok(())
}

fn run_edgar_integration_test_binary_in_docker(edgar_test_binary: String) -> anyhow::Result<()> {
    let test_binary_directory = determine_test_binary_directory()?;
    debug!("Using test binary directory: {}", test_binary_directory);

    /*
     EDGAR integration tests require:
     - Network administration capabilities (CAP_ADD NET_ADMIN)
     - Root privileges
     - iproute2 package for `ip` command
     - ca-certificates package for `update-ca-certificates` command
     */
    
    let mut docker = Command::new("docker");
    docker
        .current_dir(repo_path!())
        .arg("compose")
        .arg("--file").arg(".ci/deploy/testenv/edgar/docker-edgar-integration-test.yml")
        .arg("run")
        .env("RUN_EDGAR_NETLINK_INTEGRATION_TESTS", "true")
        .env("RUST_LOG", "info,opendut=debug")
        .arg("--rm")
        .arg("--entrypoint=")
        .arg("--volume").arg(format!("{test_binary_directory}/:/tmp/debug"))
        .arg("test")
        .arg("sh")
        .arg("-c")
        .arg(format!("/tmp/debug/{edgar_test_binary} --include-ignored --nocapture"));

    docker.run_requiring_success()?;

    Ok(())
}

fn determine_test_binary_directory() -> anyhow::Result<String> {
    let target_directory = cicero::path::target_dir();
    let test_binary_directory = target_directory.join(Arch::X86_64.triple()).join("debug").join("deps");
    let test_binary_directory = test_binary_directory
        .into_os_string().into_string()
        .map_err(|_| anyhow!("Test target directory could not be determined!"))?;
    Ok(test_binary_directory)
}


fn create_edgar_integration_test_binary() -> anyhow::Result<String> {
    let mut command = CROSS.command();
    let cargo_build_integration_test = command
        .arg("test")
        .arg("--package").arg("opendut-edgar")
        .arg("--no-run")
        .arg("--message-format=json")
        .stderr(Stdio::inherit());

    let cargo_build_integration_test = cargo_build_integration_test.output()?;
    let cargo_build = String::from_utf8_lossy(&cargo_build_integration_test.stdout);


    let test_binary = cargo_build.lines().find_map(|line| {
        let output_result = serde_json::from_str::<CargoCompilerOutput>(line);
        match output_result {
            Ok(output) => {
                output.edgar_test_binary()
            }
            Err(_) => {
                None
            }
        }
    }).ok_or(anyhow!("Could not find EDGAR test binary!"))?;

    let test_binary = test_binary
        .file_name()
        .map(|file| file.to_string_lossy().to_string())
        .ok_or(anyhow!("EDGAR test binary does not have a file name!"))?;

    Ok(test_binary)
}
