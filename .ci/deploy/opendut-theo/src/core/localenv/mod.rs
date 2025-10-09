pub(crate) mod carl_image;

use crate::core::docker::command::DockerCommand;
use crate::core::project::ProjectRootDir;
use crate::core::TestenvMode;
use anyhow::Error;
use std::path::PathBuf;
use tracing::debug;
use crate::commands::vagrant::running_in_opendut_vm;
use crate::core::dist::make_distribution_if_not_present;
use crate::core::docker::compose::{docker_compose_down, docker_compose_up_expose_ports};
use crate::core::docker::services::DockerCoreServices;
use crate::core::docker::show_error_if_unhealthy_containers_were_found;
use crate::core::localenv::carl_image::TestenvCarlImage;

pub(crate) const LOCALENV_SECRETS_PATH: &str = "./.ci/deploy/localenv/data/secrets/";
pub(crate) const LOCALENV_SECRETS_ENV_FILE: &str = "./.ci/deploy/localenv/data/secrets/.env";
pub(crate) const LOCALENV_CARL_TESTENV_TAG: &str = "testenv-latest";
pub(crate) const LOCALENV_TELEMETRY_ENABLED: &str = "OPENDUT_LOCALENV_TELEMETRY_ENABLED";

pub(crate) fn start(skip_telemetry: bool, skip_firefox: bool, expose_ports: bool, mode: &TestenvMode) -> Result<(), Error> {
    build_localenv_containers(mode)?;
    start_localenv(skip_telemetry, mode)?;

    if !skip_firefox {
        docker_compose_up_expose_ports(DockerCoreServices::Firefox.as_str(), expose_ports)?;
    }

    show_error_if_unhealthy_containers_were_found()?;

    println!("Secrets for localenv are loaded from '{}'", LOCALENV_SECRETS_ENV_FILE);
    println!("Go to OpenDuT Browser at http://localhost:3000/");

    Ok(())
}


fn start_localenv(skip_telemetry: bool, mode: &TestenvMode) -> Result<i32, Error> {
    let carl_image = TestenvCarlImage::create()?;
    let mut docker_command = DockerCommand::new();
    docker_command
        .add_localenv_args()
        .add_localenv_secrets_args()
        .env("OPENDUT_DOCKER_IMAGE_HOST", carl_image.image_host)
        .env("OPENDUT_DOCKER_IMAGE_NAMESPACE", carl_image.namespace)
        .env("OPENDUT_CARL_IMAGE_VERSION", carl_image.tag);

    if let TestenvMode::CarlDeveloperIDE = mode {
        docker_command.arg("--file")
            .arg(".ci/deploy/testenv/carl-on-host/docker-compose.localenv.override.yml");  // dev mode
        if !running_in_opendut_vm() {
            docker_command.arg("--file")
                .arg(".ci/deploy/testenv/carl-on-host/docker-compose.localenv.override.host.yml");  // dev mode
        }
    }
    if skip_telemetry {
        debug!("Disabling telemetry for localenv in testenv mode.");
        docker_command.env(LOCALENV_TELEMETRY_ENABLED, "0");
    }
    docker_command
        .arg("up")
        .arg("--detach")
        .expect_show_status("Failed to start localenv for testenv")
}

pub(crate) fn provision_localenv_secrets() -> Result<(), Error> {
    debug!("Provisioning secrets for localenv...");
    DockerCommand::new()
        .add_localenv_args()
        .arg("up")
        .arg("--build")
        .arg("provision-secrets")
        .expect_show_status("Failed to provision localenv secrets")?;

    delete_localenv_secrets()?;
    // copy secrets to host
    DockerCommand::new()
        .arg("cp")
        .arg("opendut-provision-secrets:/provision/")
        .arg(LOCALENV_SECRETS_PATH)
        .expect_show_status("Failed to copy localenv secrets.")?;
    debug!("Copied secrets to host at {}", LOCALENV_SECRETS_PATH);

    Ok(())
}

pub(crate) fn build_localenv_containers(mode: &TestenvMode) -> Result<i32, Error> {
    provision_localenv_secrets()?;
    if let TestenvMode::CarlDistribution = mode {
        make_distribution_if_not_present()?;
    }
    debug!("Building localenv containers...");
    DockerCommand::new()
        .add_localenv_args()
        .add_common_project_env()
        .arg("build")
        .expect_show_status("Failed to build localenv services")
}

pub(crate) fn destroy() -> Result<(), Error> {
    docker_compose_down(DockerCoreServices::Firefox.as_str(), true)?;
    docker_compose_down(DockerCoreServices::Edgar.as_str(), true)?;
    shutdown(true)?;
    delete_localenv_secrets()?;
    Ok(())
}

pub(crate) fn stop() -> Result<(), Error> {
    shutdown(false)?;
    Ok(())
}

fn shutdown(delete_volumes: bool) -> Result<i32, Error> {
    let mut command = DockerCommand::new();
    command.add_localenv_args();
    if delete_volumes {
        command.arg("down").arg("--volumes");
    } else {
        command.arg("down");
    }
    command.expect_show_status("Failed to shutdown localenv services.")
}

pub fn delete_localenv_secrets() -> Result<(), Error> {
    let secrets_path = PathBuf::project_path_buf().join(LOCALENV_SECRETS_PATH);
    if secrets_path.exists() {
        std::fs::remove_dir_all(&secrets_path)?;
        debug!("Deleted secrets at {:?}", &secrets_path);
    } else {
        debug!("No secrets found at {:?}", &secrets_path);
    }
    Ok(())
}
