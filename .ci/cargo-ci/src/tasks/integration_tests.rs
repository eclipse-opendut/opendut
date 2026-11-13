use anyhow::Context;
use cicero::command_exit_ok::CommandExitOk;
use cicero::path::repo_path;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tracing::debug;
use crate::core::commands::CROSS;

/// Run integration tests of EDGAR
#[derive(Debug, clap::Parser)]
pub struct IntegrationTestCli {
}

impl IntegrationTestCli {
    pub fn run(self) -> anyhow::Result<()> { test(self) }
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
pub fn test(_params: IntegrationTestCli) -> anyhow::Result<()> {
    let edgar_test_binary = create_edgar_integration_test_binary()?;
    debug!("EDGAR test binary: {:?}", edgar_test_binary);

    run_edgar_integration_test_binary_in_docker(edgar_test_binary)?;

    Ok(())
}

fn run_edgar_integration_test_binary_in_docker(test_binary: PathBuf) -> anyhow::Result<()> {
    let test_binary_directory = test_binary.parent().unwrap().display();
    debug!("Using test binary directory: {test_binary_directory}");

    let test_binary = test_binary.display();

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
        .arg("--volume").arg(format!("{test_binary_directory}:{test_binary_directory}"))
        .arg("test")
        .arg("sh")
        .arg("-c")
        .arg(format!("{test_binary} --include-ignored --nocapture"));

    docker.status_exit_ok()?;

    Ok(())
}

fn create_edgar_integration_test_binary() -> anyhow::Result<PathBuf> {
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
    }).context("Could not find EDGAR test binary!")?;

    let test_binary = repo_path!().join(
        test_binary.strip_prefix("/")? //path starts with `/target/...`
    );

    Ok(test_binary)
}
