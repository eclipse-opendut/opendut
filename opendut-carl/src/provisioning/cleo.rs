use std::fmt::{Display, Formatter};
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path};
use config::Config;
use flate2::Compression;
use flate2::write::GzEncoder;
use pem::{encode_config, EncodeConfig, LineEnding, Pem};
use tracing::log::warn;
use crate::util::{CLEO_TARGET_DIRECTORY, CleoArch};

const CA_CERTIFICATE_FILE_NAME: &str = "ca.pem";

pub struct CleoScript {
    carl_host: String,
    carl_port: u16,
    oidc_enabled: bool,
    issuer_url: String,
}

impl CleoScript {
    pub fn from_setting(settings: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            carl_host: settings.get_string("network.remote.host")?,
            carl_port: settings.get_int("network.remote.port")? as u16,
            oidc_enabled: settings.get_bool("network.oidc.enabled")?,
            issuer_url: settings.get_string("network.oidc.client.issuer.url")?,
        })
    }
}

impl Display for CleoScript {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format!(r#"#!/bin/bash

DIR_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
CERT_PATH=$DIR_PATH/{}

export OPENDUT_CLEO_NETWORK_OIDC_CLIENT_SCOPES=
export OPENDUT_CLEO_NETWORK_TLS_DOMAIN_NAME_OVERRIDE={}
export OPENDUT_CLEO_NETWORK_TLS_CA=$CERT_PATH
export OPENDUT_CLEO_NETWORK_CARL_HOST={}
export OPENDUT_CLEO_NETWORK_CARL_PORT={}
export OPENDUT_CLEO_NETWORK_OIDC_ENABLED={}
export OPENDUT_CLEO_NETWORK_OIDC_CLIENT_ISSUER_URL={}
export SSL_CERT_FILE=$CERT_PATH"#,
                    CA_CERTIFICATE_FILE_NAME,
                    self.carl_host,
                    self.carl_host,
                    self.carl_port,
                    self.oidc_enabled,
                    self.issuer_url
            ).as_str()
        )
    }
}

pub fn create_cleo_install_script(
    ca: Pem,
    cleo_install_path: &Path,
    cleo_script: CleoScript,
) -> anyhow::Result<()> {
    const SET_ENVIRONMENT_VARIABLES_SCRIPT_NAME: &str = "set-env-var.sh";
    const PERMISSION_CODE: u32 = 0o775;

    let ca_file_path = cleo_install_path.join(CLEO_TARGET_DIRECTORY).join(CA_CERTIFICATE_FILE_NAME);
    std::fs::write(&ca_file_path, encode_config(&ca, EncodeConfig::new().set_line_ending(LineEnding::LF)))?;
    let ca_file = &mut File::open(ca_file_path)?;

    let script_path = cleo_install_path.join(CLEO_TARGET_DIRECTORY).join(SET_ENVIRONMENT_VARIABLES_SCRIPT_NAME);

    match std::fs::write(
        &script_path,
        format!("{}", cleo_script)
    )  {
        Ok(_) => {}
        Err(error) => { warn!("Could not write {}: {}", SET_ENVIRONMENT_VARIABLES_SCRIPT_NAME, error) }
    };

    let mut permissions = std::fs::metadata(&script_path)?.permissions();
    permissions.set_mode(PERMISSION_CODE);
    std::fs::set_permissions(&script_path, permissions)?;

    let script_file = &mut File::open(script_path)?;

    for arch in CleoArch::arch_iterator() {
        let cleo_file = cleo_install_path.join(CLEO_TARGET_DIRECTORY).join(&arch.file_name());
        let file_name = format!("{}.tar.gz", &arch.file_name());
        let file_path = cleo_install_path.join(CLEO_TARGET_DIRECTORY).join(&file_name);

        match File::create(file_path) {
            Ok(tar_gz) => {
                let enc = GzEncoder::new(tar_gz, Compression::default());
                let mut tar = tar::Builder::new(enc);
                tar.append_file(
                    &arch.file_name(),
                    &mut File::open(&cleo_file)?
                )?;
                tar.append_file(SET_ENVIRONMENT_VARIABLES_SCRIPT_NAME, script_file)?;
                tar.append_file(CA_CERTIFICATE_FILE_NAME, ca_file)?;
                tar.into_inner()?;
            }
            Err(_) => {
                warn!("Could not create {}.tar.gz", &file_name);
            }
        };
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::path::{PathBuf};
    use std::str::FromStr;
    use assert_fs::fixture::{FileTouch, PathChild};
    use assert_fs::TempDir;
    use googletest::assert_that;
    use googletest::prelude::eq;
    use pem::Pem;
    use crate::provisioning::cleo::{CleoScript, create_cleo_install_script};
    use crate::util::{CLEO_TARGET_DIRECTORY, CleoArch};

    #[tokio::test()]
    async fn creating_cleo_install_script_succeeds() -> anyhow::Result<()> {

        let temp = TempDir::new().unwrap();
        let dir = temp.child(CLEO_TARGET_DIRECTORY);
        std::fs::create_dir_all(dir).unwrap();

        for arch in CleoArch::arch_iterator() {
            let file = temp.child(PathBuf::from(CLEO_TARGET_DIRECTORY).join(arch.file_name()));
            file.touch().unwrap();
        }

        let cert = match Pem::from_str(include_str!("../../../resources/development/tls/insecure-development-ca.pem")) {
            Ok(cert) => { cert }
            Err(_) => { panic!("Not a valid certificate!") }
        };

        create_cleo_install_script(
            cert,
            &temp.to_path_buf(),
            CleoScript {
                carl_host: "carl".to_string(),
                carl_port: 443,
                oidc_enabled: true,
                issuer_url: "https://keycloak/realms/opendut/".to_string(),
            }
        )?;

        for arch in CleoArch::arch_iterator() {
            assert_that!(temp.join("opendut-cleo").join(format!("{}.tar.gz",arch.file_name())).exists(), eq(true));
        }

        Ok(())
    }
}
