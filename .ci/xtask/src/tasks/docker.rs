use std::process::Command;
use std::str::FromStr;

use crate::core::constants::workspace_dir;
use crate::core::types::Package;
use crate::core::util::RunRequiringSuccess;

/// A Docker tag
#[derive(Clone, Debug)]
pub struct DockerTag(pub String);

impl FromStr for DockerTag {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

/// Build and publish a Docker image
#[derive(Debug, clap::Parser)]
pub struct DockerCli {
    /// Custom docker tag instead of the default version
    #[arg(long)]
    pub tag: Option<DockerTag>,
}

const OPENDUT_DOCKER_IMAGE_HOST: &str = "ghcr.io";
const OPENDUT_DOCKER_IMAGE_NAMESPACE: &str = "eclipse-opendut";

fn carl_container_uri(tag: &Option<DockerTag>) -> String {
    let image_host = std::env::var("OPENDUT_DOCKER_IMAGE_HOST").unwrap_or(OPENDUT_DOCKER_IMAGE_HOST.to_string());
    let image_namespace = std::env::var("OPENDUT_DOCKER_IMAGE_NAMESPACE").unwrap_or(OPENDUT_DOCKER_IMAGE_NAMESPACE.to_string());
    let version = match tag {
        None => { crate::build::PKG_VERSION }
        Some(tag) => {
            tag.0.as_str()
        }
    };
    let image_uri = format!("{}/{}/{}:{}", image_host, image_namespace, Package::Carl.ident(), version);
    image_uri
}

pub fn build_carl_docker_image(tag: Option<DockerTag>) -> crate::Result {
    let image_version_build_arg = format!("VERSION={}", crate::build::PKG_VERSION);
    let now = chrono::Utc::now().naive_utc();

    // https://github.com/opencontainers/image-spec/blob/main/annotations.md
    let source = format!("org.opencontainers.image.source={}", crate::core::metadata::repository_url());
    let url = format!("org.opencontainers.image.url={}", carl_container_uri(&tag));
    let version = format!("org.opencontainers.image.version={}", crate::build::PKG_VERSION);
    let created = format!("org.opencontainers.image.created={}", now);
    let revision = format!("org.opencontainers.image.revision={}", crate::build::COMMIT_HASH);

    Command::new("docker")
        .current_dir(workspace_dir())
        .args([
            "build",
            "--file",
            ".ci/docker/carl/Dockerfile",
            "--build-arg",
            &image_version_build_arg,
            "--label", &source,
            "--label", &url,
            "--label", &version,
            "--label", &created,
            "--label", &revision,
            "--tag",
            &carl_container_uri(&tag),
            ".",
        ])
        .run_requiring_success()?;
    Ok(())
}

pub fn publish_carl_docker_image(tag: Option<DockerTag>) -> crate::Result {
    Command::new("docker")
        .current_dir(workspace_dir())
        .args(["push", &carl_container_uri(&tag)])
        .run_requiring_success()?;
    Ok(())
}
