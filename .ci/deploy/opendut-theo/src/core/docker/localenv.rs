use std::path::PathBuf;
use anyhow::Error;
use chrono::Utc;
use tracing::debug;
use crate::core::docker::DockerCommand;
use crate::core::project::ProjectRootDir;

pub(crate) const LOCALENV_SECRETS_PATH: &str = "./.ci/deploy/localenv/data/secrets/";
pub(crate) const LOCALENV_SECRETS_ENV_FILE: &str = "./.ci/deploy/localenv/data/secrets/.env";

pub struct TestenvCarlImage {
    pub image_host: String,
    pub namespace: String,
    pub tag: String,
}

impl TestenvCarlImage {
    pub fn new(carl_version: &str) -> Self {
        let image_host = "opendut-testenv-docker-registry".to_string();
        let namespace = "opendut-docker-namespace".to_string();
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let tag = format!("{}-{}", carl_version, timestamp);
        Self {
            image_host,
            namespace,
            tag,
        }
    }

    pub fn full_image_name(&self) -> String {
        // Example:
        // "${OPENDUT_DOCKER_IMAGE_HOST:-ghcr.io}/${OPENDUT_DOCKER_IMAGE_NAMESPACE:-eclipse-opendut}/opendut-carl:${OPENDUT_CARL_IMAGE_VERSION:-0.7.0}"
        format!("{}/{}/opendut-carl:{}", self.image_host, self.namespace, self.tag)
    }
}

pub(crate) fn docker_localenv_shutdown(delete_volumes: bool) -> Result<i32, Error> {
    let mut command = DockerCommand::new();
    command.add_localenv_args();
    if delete_volumes {
        command.arg("down").arg("--volumes");
    } else {
        command.arg("down");
    }
    command.expect_status("Failed to shutdown localenv services.")
}

pub(crate) fn delete_localenv_secrets() -> Result<(), Error> {
    let secrets_path = PathBuf::project_path_buf().join(LOCALENV_SECRETS_PATH);
    if secrets_path.exists() {
        std::fs::remove_dir_all(&secrets_path)?;
        debug!("Deleted secrets at {:?}", &secrets_path);
    } else {
        debug!("No secrets found at {:?}", &secrets_path);
    }
    Ok(())
}