use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::Context;
use tempfile::TempDir;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use opendut_util::pem::Pem;

pub mod script;

use script::CleoScript;
use crate::http::router::arch::{CLEO_IDENTIFIER, CleoArch};

const CA_CERTIFICATE_FILE_NAME: &str = "ca.pem";
const SET_ENVIRONMENT_VARIABLES_SCRIPT_NAME: &str = "cleo-cli.sh";
const PERMISSION_CODE_SCRIPT: u32 = 0o775;
const PERMISSION_CODE_CA: u32 = 0o644;

pub fn create_cleo_install_script(
    ca: &Pem,
    carl_install_directory: &Path,
    cleo_script: &CleoScript,
) -> anyhow::Result<()> {

    for arch in CleoArch::arch_iterator() {
        let cleo_tar_file = carl_install_directory.join(CLEO_IDENTIFIER).join(format!("{}-{}.tar.gz", arch.distribution_name(), crate::app_info::PKG_VERSION));
        add_file_to_archive(
            ca,
            &cleo_tar_file,
            cleo_script,
        )?;
    }

    Ok(())
}

fn add_file_to_archive(
    ca: &Pem,
    cleo_tar_file: &Path,
    cleo_script: &CleoScript,
) -> anyhow::Result<()> {

    let unpack_dir = {
        let unpack_dir = TempDir::new()?;
        let mut archive = tar::Archive::new(GzDecoder::new(File::open(cleo_tar_file)?));
        archive.unpack(&unpack_dir)?;
        unpack_dir
    };

    let tar_gz = File::create(cleo_tar_file)
        .context(format!("Could not create path '{}' for CLEO archive.", cleo_tar_file.display()))?;

    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);

    tar.append_dir_all("", unpack_dir)?;
    tar.append_custom_data(
        &cleo_script.build_script(),
        PathBuf::from(CLEO_IDENTIFIER).join(SET_ENVIRONMENT_VARIABLES_SCRIPT_NAME),
        PERMISSION_CODE_SCRIPT
    )?;
    tar.append_custom_data(
        &ca.to_string(),
        PathBuf::from(CLEO_IDENTIFIER).join(CA_CERTIFICATE_FILE_NAME),
        PERMISSION_CODE_CA
    )?;
    tar.into_inner()?.finish()?;

    Ok(())
}



pub trait AppendCustomData {
    fn append_custom_data(&mut self, data: &str, file_name: PathBuf, mode: u32) -> std::io::Result<()>;
}
impl AppendCustomData for tar::Builder<GzEncoder<File>> {
    fn append_custom_data(&mut self, data: &str, file_name: PathBuf, mode: u32) -> std::io::Result<()> {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(mode);
        header.set_cksum();
        self.append_data(&mut header, file_name, data.as_bytes())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::fs;
    use std::fs::File;
    use std::str::FromStr;
    use assert_fs::assert::PathAssert;

    use assert_fs::fixture::PathChild;
    use assert_fs::prelude::PathCreateDir;
    use assert_fs::TempDir;
    use flate2::Compression;
    use flate2::read::GzDecoder;
    use flate2::write::GzEncoder;
    use predicates::path;

    use crate::http::router::arch::CLEO_IDENTIFIER;

    #[tokio::test()]
    async fn creating_cleo_install_script_succeeds() -> anyhow::Result<()> {

        let temp = TempDir::new().unwrap();
        let tar_file = temp.child("file.tar.gz");

        let tar_gz = File::create(&tar_file).unwrap();

        let mut tar_gz = tar::Builder::new(
            GzEncoder::new(tar_gz, Compression::default())
        );

        let cleo_dir = temp.child(CLEO_IDENTIFIER);
        fs::create_dir_all(&cleo_dir).expect("Unable to create dir.");

        let archived_file = cleo_dir.child("test.txt");
        fs::write(&archived_file, "TEST")?;

        tar_gz.append_dir_all(CLEO_IDENTIFIER, &cleo_dir)?;
        tar_gz.into_inner()?.finish()?;

        let cert = Pem::from_str(include_str!("../../../../resources/development/tls/insecure-development-ca.pem"))
            .expect("Not a valid certificate!"); 

        let cleo_script = CleoScript {
            carl_host: "carl".to_string(),
            carl_port: 443,
            oidc_enabled: true,
            issuer_url: "https://keycloak/realms/opendut/".to_string(),
        };

        add_file_to_archive(
            &cert,
            &tar_file,
            &cleo_script,
        )?;

        assert!(tar_file.exists());

        let unpack_dir = {
            let unpack_dir = temp.child("unpack_dir");
            unpack_dir.create_dir_all()?;
            let mut archive = tar::Archive::new(GzDecoder::new(File::open(tar_file)?));
            archive.unpack(&unpack_dir)?;
            unpack_dir
        };

        let existing_test_file = unpack_dir.child(CLEO_IDENTIFIER).child("test.txt");
        existing_test_file.assert(path::is_file());

        Ok(())
    }
}
