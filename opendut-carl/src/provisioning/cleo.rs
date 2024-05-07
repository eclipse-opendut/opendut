use std::fs::File;
use std::path::Path;

use anyhow::Context;
use flate2::Compression;
use flate2::write::GzEncoder;
use pem::Pem;

use crate::provisioning::cleo_script::CleoScript;
use crate::util::{CLEO_IDENTIFIER, CleoArch};

pub const CA_CERTIFICATE_FILE_NAME: &str = "ca.pem";

pub fn create_cleo_install_script(
    ca: Pem,
    carl_install_directory: &Path,
    cleo_script: CleoScript,
) -> anyhow::Result<()> {
    const SET_ENVIRONMENT_VARIABLES_SCRIPT_NAME: &str = "cleo-cli.sh";
    const PERMISSION_CODE_SCRIPT: u32 = 0o775;
    const PERMISSION_CODE_CA: u32 = 0o644;

    for arch in CleoArch::arch_iterator() {
        let cleo_file = carl_install_directory.join(CLEO_IDENTIFIER).join(&arch.name()).join(CLEO_IDENTIFIER);
        let file_name = format!("{}.tar.gz", &arch.name());
        let file_path = carl_install_directory.join(CLEO_IDENTIFIER).join(&file_name);

        let tar_gz = File::create(&file_path)
            .context(format!("Could not create path '{}' for CLEO archive.", &file_path.display()))?;

        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = tar::Builder::new(enc);
        tar.append_file(
            &arch.name(),
            &mut File::open(&cleo_file)
                .context(format!("Failed to open CLEO executable file '{}'", cleo_file.display()))?
        )?;
        tar.append_custom_data(&cleo_script.build_script(arch), SET_ENVIRONMENT_VARIABLES_SCRIPT_NAME, PERMISSION_CODE_SCRIPT)?;
        tar.append_custom_data(&ca.to_string(), CA_CERTIFICATE_FILE_NAME, PERMISSION_CODE_CA)?;
        tar.into_inner()?;
    }

    Ok(())
}

pub trait AppendCustomData {
    fn append_custom_data(&mut self, data: &str, file_name: &str, mode: u32) -> std::io::Result<()>;
}
impl AppendCustomData for tar::Builder<GzEncoder<File>> {
    fn append_custom_data(&mut self, data: &str, file_name: &str, mode: u32) -> std::io::Result<()> {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.as_bytes().len() as u64);
        header.set_mode(mode);
        header.set_cksum();
        self.append_data(&mut header, file_name, data.as_bytes())
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use std::str::FromStr;

    use assert_fs::fixture::{FileTouch, PathChild};
    use assert_fs::TempDir;
    use googletest::assert_that;
    use googletest::prelude::eq;
    use pem::Pem;

    use crate::provisioning::cleo::{CleoScript, create_cleo_install_script};
    use crate::util::{CLEO_IDENTIFIER, CleoArch};

    #[tokio::test()]
    async fn creating_cleo_install_script_succeeds() -> anyhow::Result<()> {

        let temp = TempDir::new().unwrap();
        let dir = temp.child(CLEO_IDENTIFIER);
        std::fs::create_dir_all(dir).unwrap();

        for arch in CleoArch::arch_iterator() {
            let cleo_dir = temp.child(PathBuf::from(CLEO_IDENTIFIER).join(arch.name()));
            std::fs::create_dir_all(cleo_dir).unwrap();
            
            let file = temp.child(PathBuf::from(CLEO_IDENTIFIER).join(arch.name()).join(CLEO_IDENTIFIER));
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
            assert_that!(temp.join("opendut-cleo").join(format!("{}.tar.gz",arch.name())).exists(), eq(true));
        }

        Ok(())
    }
}
