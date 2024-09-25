use crate::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;
use async_trait::async_trait;
use tracing::debug;

use opendut_types::util::net::Certificate;

use crate::setup::{constants, util};
use crate::common::task::{Success, Task, TaskFulfilled};
use crate::setup::util::{CommandRunner, DefaultCommandRunner};

pub struct WriteCaCertificate {
    pub certificate: Certificate,
    pub carl_ca_certificate_path: PathBuf,
    pub os_cert_store_ca_certificate_path: PathBuf,
    pub checksum_carl_ca_certificate_file: PathBuf,
    pub checksum_os_cert_store_ca_certificate_file: PathBuf,
    pub command_runner: Box<dyn CommandRunner>,
}

#[async_trait]
impl Task for WriteCaCertificate {

    fn description(&self) -> String {
        String::from("Write CA Certificates")
    }

    async fn check_fulfilled(&self) -> anyhow::Result<TaskFulfilled> {
        let installed_carl_checksum_file = &self.checksum_carl_ca_certificate_file;
        let installed_os_cert_store_checksum_file = &self.checksum_os_cert_store_ca_certificate_file;

        let (installed_carl_checksum, installed_os_cert_store_checksum) = {
            if installed_carl_checksum_file.exists()
            && installed_os_cert_store_checksum_file.exists() {
                (
                    fs::read(installed_carl_checksum_file)?,
                    fs::read(installed_os_cert_store_checksum_file)?,
                )
            }
            else if self.carl_ca_certificate_path.exists()
            && self.os_cert_store_ca_certificate_path.exists() {
                debug!("No previous certificate checksum files exist, but certificate files found. Calculating checksum by reading them.");
                (
                    util::checksum::file(&self.carl_ca_certificate_path)?,
                    util::checksum::file(&self.os_cert_store_ca_certificate_path)?,
                )
            } else {
                debug!("No previous certificate checksum files nor certificate files exist. Task needs execution.");
                return Ok(TaskFulfilled::No);
            }
        };

        let provided_certificate_checksum = util::checksum::string(
            self.certificate.encode_as_string()
        )?;

        if installed_carl_checksum == provided_certificate_checksum
        && installed_os_cert_store_checksum == provided_certificate_checksum {
            Ok(TaskFulfilled::Yes)
        } else {
            debug!("Previous certificate checksum files exist, but do not match. Task needs execution.");
            Ok(TaskFulfilled::No)
        }
    }

    async fn execute(&self) -> anyhow::Result<Success> {
        let carl_ca_certificate_path = &self.carl_ca_certificate_path;

        write_carl_certificate(&self.certificate, carl_ca_certificate_path, &self.checksum_carl_ca_certificate_file)?;

        write_os_cert_store_certificate(carl_ca_certificate_path, &self.os_cert_store_ca_certificate_path, &self.checksum_os_cert_store_ca_certificate_file, self.command_runner.as_ref())?; //TODO this certificate doesn't have to be the same as for CARL and should instead be retrieved from CARL after the initial connection

        Ok(Success::default())
    }
}

impl WriteCaCertificate {
    pub fn with_certificate(certificate: Certificate) -> Self {
        Self {
            certificate,
            carl_ca_certificate_path: constants::default_carl_ca_certificate_path(),
            os_cert_store_ca_certificate_path: constants::default_os_cert_store_ca_certificate_path(),
            checksum_carl_ca_certificate_file: constants::default_checksum_carl_ca_certificate_file(),
            checksum_os_cert_store_ca_certificate_file: constants::default_checksum_os_cert_store_ca_certificate_file(),
            command_runner: Box::new(DefaultCommandRunner),
        }
    }
}

fn write_carl_certificate(new_certificate: &Certificate, carl_ca_certificate_path: &Path, checksum_carl_ca_certificate_file: &Path) -> anyhow::Result<()> {

    let carl_ca_certificate_dir = carl_ca_certificate_path.parent().unwrap();
    fs::create_dir_all(carl_ca_certificate_dir)
        .context(format!("Unable to create path {:?}", carl_ca_certificate_dir))?;
    
    fs::write(
        carl_ca_certificate_path,
        new_certificate.encode_as_string()
    ).context(format!(
        "Write CA certificate was not successful at location {:?}", carl_ca_certificate_path
    ))?;

    let checksum = util::checksum::file(carl_ca_certificate_path)?;
    let checksum_unpack_file = checksum_carl_ca_certificate_file;
    fs::create_dir_all(checksum_unpack_file.parent().unwrap())?;
    fs::write(checksum_unpack_file, checksum)
        .context(format!("Writing checksum for carl ca certificate to '{}'.", checksum_unpack_file.display()))?;
    
    Ok(())
}

fn write_os_cert_store_certificate(
    carl_ca_certificate_path: &Path, 
    os_cert_store_ca_certificate_path: &Path,
    checksum_os_cert_store_ca_certificate_file: &Path,
    command_runner: &dyn CommandRunner
) -> anyhow::Result<()> {

    let os_cert_store_ca_certificate_dir = os_cert_store_ca_certificate_path.parent().unwrap();
    fs::create_dir_all(os_cert_store_ca_certificate_dir)
        .context(format!("Unable to create path {:?}", os_cert_store_ca_certificate_dir))?;

    fs::copy(
        carl_ca_certificate_path,
        os_cert_store_ca_certificate_path,
    ).context(format!(
        "Copying CA certificate from {:?} to {:?} was not possible.", carl_ca_certificate_path, os_cert_store_ca_certificate_path
    ))?;

    let update_ca_certificates = which::which("update-ca-certificates")
        .context(String::from("No command `update-ca-certificates` found. Ensure your system provides this command."))?;

    command_runner.run(
        &mut Command::new(update_ca_certificates) //Update OS certificate store, as NetBird and reqwest (for result uploading to WebDAV) reads from there
    ).context("update-ca-certificates could not be executed successfully!")?;

    let checksum = util::checksum::file(os_cert_store_ca_certificate_path)?;
    let checksum_unpack_file = checksum_os_cert_store_ca_certificate_file;
    fs::create_dir_all(checksum_unpack_file.parent().unwrap())?;
    fs::write(checksum_unpack_file, checksum)
        .context(format!("Writing checksum for OS cert store ca certificate to '{}'.", checksum_unpack_file.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use assert_fs::prelude::*;
    use assert_fs::TempDir;

    use opendut_types::util::net::Certificate;

    use crate::common::task::{Task, TaskFulfilled};
    use crate::setup::tasks::WriteCaCertificate;
    use crate::setup::util;
    use crate::setup::util::NoopCommandRunner;

    #[tokio::test]
    async fn should_report_task_as_fulfilled_after_execution() -> anyhow::Result<()> {
        let temp = TempDir::new()?;

        let carl_ca_certificate_path = temp.child("ca.pem");
        let os_cert_store_ca_certificate_path = temp.child("opendut-ca.crt");

        let checksum_carl_ca_certificate_file = temp.child("ca.pem.checksum");
        let checksum_os_cert_store_ca_certificate_file = temp.child("opendut-ca.crt.checksum");

        let pem_string = PEM_STRING_1;

        let task = WriteCaCertificate {
            certificate: Certificate::from_str(pem_string)?,
            carl_ca_certificate_path: carl_ca_certificate_path.to_path_buf(),
            os_cert_store_ca_certificate_path: os_cert_store_ca_certificate_path.to_path_buf(),
            checksum_carl_ca_certificate_file: checksum_carl_ca_certificate_file.to_path_buf(),
            checksum_os_cert_store_ca_certificate_file: checksum_os_cert_store_ca_certificate_file.to_path_buf(),
            command_runner: Box::new(NoopCommandRunner),
        };

        assert_eq!(task.check_fulfilled().await?, TaskFulfilled::No);
        task.execute().await?;
        assert_eq!(task.check_fulfilled().await?, TaskFulfilled::Yes);

        Ok(())
    }

    #[tokio::test]
    async fn should_report_task_as_unfulfilled_when_checksums_dosssnt_match() -> anyhow::Result<()> {
        let temp = TempDir::new()?;

        let carl_ca_certificate_path = temp.child("ca.pem");
        let os_cert_store_ca_certificate_path = temp.child("opendut-ca.crt");

        let checksum_carl_ca_certificate_file = temp.child("ca.pem.checksum");
        let checksum_os_cert_store_ca_certificate_file = temp.child("opendut-ca.crt.checksum");

        let stored_pem = PEM_STRING_1;
        let new_pem = PEM_STRING_2;

        carl_ca_certificate_path.write_str(stored_pem)?;
        os_cert_store_ca_certificate_path.write_str(stored_pem)?;

        let checksum_carl_os_cert_store_cert = util::checksum::file(&carl_ca_certificate_path)?;
        checksum_carl_ca_certificate_file.write_binary(&checksum_carl_os_cert_store_cert)?;
        checksum_os_cert_store_ca_certificate_file.write_binary(&checksum_carl_os_cert_store_cert)?;


        let task = WriteCaCertificate {
            certificate: Certificate::from_str(new_pem)?,
            carl_ca_certificate_path: carl_ca_certificate_path.to_path_buf(),
            os_cert_store_ca_certificate_path: os_cert_store_ca_certificate_path.to_path_buf(),
            checksum_carl_ca_certificate_file: checksum_carl_ca_certificate_file.to_path_buf(),
            checksum_os_cert_store_ca_certificate_file: checksum_os_cert_store_ca_certificate_file.to_path_buf(),
            command_runner: Box::new(NoopCommandRunner),
        };

        assert_eq!(task.check_fulfilled().await?, TaskFulfilled::No);

        Ok(())
    }

    #[tokio::test]
    async fn should_report_task_as_fulfilled_when_checksums_match() -> anyhow::Result<()> {
        let temp = TempDir::new()?;

        let carl_ca_certificate_path = temp.child("ca.pem");
        let os_cert_store_ca_certificate_path = temp.child("opendut-ca.crt");

        let checksum_carl_ca_certificate_file = temp.child("ca.pem.checksum");
        let checksum_os_cert_store_ca_certificate_file = temp.child("opendut-ca.crt.checksum");

        let pem_string = PEM_STRING_1;

        carl_ca_certificate_path.write_str(pem_string)?;
        os_cert_store_ca_certificate_path.write_str(pem_string)?;

        let checksum_carl_os_cert_store_cert = util::checksum::file(&carl_ca_certificate_path)?;
        let checksum_string = checksum_carl_os_cert_store_cert.clone();
        checksum_carl_ca_certificate_file.write_binary(&checksum_string)?;
        checksum_os_cert_store_ca_certificate_file.write_binary(&checksum_string)?;


        let task = WriteCaCertificate {
            certificate: Certificate::from_str(pem_string)?,
            carl_ca_certificate_path: carl_ca_certificate_path.to_path_buf(),
            os_cert_store_ca_certificate_path: os_cert_store_ca_certificate_path.to_path_buf(),
            checksum_carl_ca_certificate_file: checksum_carl_ca_certificate_file.to_path_buf(),
            checksum_os_cert_store_ca_certificate_file: checksum_os_cert_store_ca_certificate_file.to_path_buf(),
            command_runner: Box::new(NoopCommandRunner),
        };


        assert_eq!(task.check_fulfilled().await?, TaskFulfilled::Yes);

        Ok(())
    }

    #[tokio::test]
    async fn should_report_task_as_fulfilled_when_checksums_dont_exist_but_the_certificate_files_on_disk_match() -> anyhow::Result<()> { //useful for placing the certificate files onto disk for an externally automated setup of EDGAR
        let temp = TempDir::new()?;

        let carl_ca_certificate_path = temp.child("ca.pem");
        let os_cert_store_ca_certificate_path = temp.child("opendut-ca.crt");

        let checksum_carl_ca_certificate_file = temp.child("ca.pem.checksum");
        let checksum_os_cert_store_ca_certificate_file = temp.child("opendut-ca.crt.checksum");

        let pem_string = PEM_STRING_1;

        carl_ca_certificate_path.write_str(pem_string)?;
        os_cert_store_ca_certificate_path.write_str(pem_string)?;

        let task = WriteCaCertificate {
            certificate: Certificate::from_str(pem_string)?,
            carl_ca_certificate_path: carl_ca_certificate_path.to_path_buf(),
            os_cert_store_ca_certificate_path: os_cert_store_ca_certificate_path.to_path_buf(),
            checksum_carl_ca_certificate_file: checksum_carl_ca_certificate_file.to_path_buf(),
            checksum_os_cert_store_ca_certificate_file: checksum_os_cert_store_ca_certificate_file.to_path_buf(),
            command_runner: Box::new(NoopCommandRunner),
        };

        assert_eq!(task.check_fulfilled().await?, TaskFulfilled::Yes);

        Ok(())
    }

    const PEM_STRING_1: &str = "-----BEGIN RSA PUBLIC KEY-----
MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
-----END RSA PUBLIC KEY-----
";

    const PEM_STRING_2: &str = "-----BEGIN RSA PUBLIC KEY-----
MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
-----END RSA PUBLIC KEY-----
";
}
