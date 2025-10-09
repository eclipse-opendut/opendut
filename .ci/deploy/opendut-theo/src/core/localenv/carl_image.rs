use std::path::PathBuf;
use anyhow::Error;
use crate::core::docker::command::DockerCommand;
use crate::core::localenv::LOCALENV_CARL_TESTENV_TAG;
use crate::core::project::ProjectRootDir;
use crate::core::TARGET_TRIPLE;
use crate::core::util::file_modified_time_in_seconds;

pub struct TestenvCarlImage {
    pub image_host: String,
    pub namespace: String,
    pub carl_version: String,
    pub carl_dist_file_timestamp: u64,
    pub tag: String,
}

impl TestenvCarlImage {
    pub fn create() -> Result<Self, Error> {
        let carl_version = crate::core::metadata::get_package_version("opendut-carl");
        let carl_dist_path = PathBuf::project_dist_path_buf().join(TARGET_TRIPLE).join(format!("opendut-carl-{TARGET_TRIPLE}-{carl_version}.tar.gz"));
        if carl_dist_path.exists() {
            let carl_dist_file_timestamp = file_modified_time_in_seconds(&carl_dist_path)?;

            let image_host = "opendut-testenv-docker-registry".to_string();
            let namespace = "opendut-docker-namespace".to_string();

            let carl_image = Self {
                image_host,
                namespace,
                carl_version,
                carl_dist_file_timestamp,
                tag: LOCALENV_CARL_TESTENV_TAG.to_string(),
            };
            carl_image.build()?;

            Ok(carl_image)
        } else {
            Err(anyhow::anyhow!("Carl distribution not found at {:?}. Please run 'cargo make distribution' first.", carl_dist_path))
        }
    }

    fn build(&self) -> Result<(), Error> {
        let carl_docker_file = PathBuf::project_path_buf().join(".ci/docker/carl/Dockerfile");

        DockerCommand::new()
            .arg("build")
            .arg("--file").arg(&carl_docker_file)
            .arg("--label").arg(format!("dev.eclipse.opendut.carl.version={}", self.carl_version))
            .arg("--label").arg(format!("dev.eclipse.opendut.carl.dist_timestamp={}", self.carl_dist_file_timestamp))
            .arg("--build-arg").arg(format!("VERSION={}", self.carl_version))
            .arg("--tag").arg(self.full_image_name())
            .arg(".")
            .expect_show_status("Failed to build docker image")?;

        Ok(())
    }

    fn full_image_name(&self) -> String {
        format!("{}/{}/opendut-carl:{}", self.image_host, self.namespace, LOCALENV_CARL_TESTENV_TAG)
    }
}