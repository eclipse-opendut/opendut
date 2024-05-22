use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;
use pem::Pem;

use opendut_types::util::net::Certificate;

use crate::setup::{constants, util};
use crate::setup::task::{Success, Task, TaskFulfilled};
use crate::setup::util::{CommandRunner, DefaultCommandRunner};

pub struct WriteCaCertificate {
    pub certificate: Certificate,
    pub carl_ca_certificate_path: PathBuf,
    pub netbird_ca_certificate_path: PathBuf,
    pub checksum_carl_ca_certificate_file: PathBuf,
    pub checksum_netbird_ca_certificate_file: PathBuf,
    pub command_runner: Box<dyn CommandRunner>,
}

impl Task for WriteCaCertificate {

    fn description(&self) -> String {
        String::from("Write CA Certificates")
    }

    fn check_fulfilled(&self) -> anyhow::Result<TaskFulfilled> {
        let unpacked_carl_checksum_file = &self.checksum_carl_ca_certificate_file;
        let unpacked_netbird_checksum_file = &self.checksum_netbird_ca_certificate_file;

        if unpacked_carl_checksum_file.exists() && unpacked_netbird_checksum_file.exists() {
            let carl_installed_digest = fs::read(unpacked_carl_checksum_file)?;
            let carl_distribution_digest = util::file_checksum(&self.carl_ca_certificate_path)?;
            let netbird_installed_digest = fs::read(unpacked_netbird_checksum_file)?;
            let netbird_distribution_digest = util::file_checksum(&self.netbird_ca_certificate_path)?;

            if carl_installed_digest == carl_distribution_digest
                && netbird_installed_digest == netbird_distribution_digest {
                return Ok(TaskFulfilled::Yes);
            }
        }
        Ok(TaskFulfilled::No)
    }

    fn execute(&self) -> anyhow::Result<Success> {
        let Certificate(new_certificate) = Clone::clone(&self.certificate);

        let carl_ca_certificate_path = &self.carl_ca_certificate_path;

        write_carl_certificate(new_certificate, carl_ca_certificate_path, &self.checksum_carl_ca_certificate_file)?;

        write_netbird_certificate(carl_ca_certificate_path, &self.netbird_ca_certificate_path, &self.checksum_netbird_ca_certificate_file, self.command_runner.as_ref())?; //TODO this certificate doesn't have to be the same as for CARL and should instead be retrieved from CARL after the initial connection

        Ok(Success::default())
    }
}

impl WriteCaCertificate {
    pub fn with_certificate(certificate: Certificate) -> Self {
        Self {
            certificate,
            carl_ca_certificate_path: constants::default_carl_ca_certificate_path(),
            netbird_ca_certificate_path: constants::default_netbird_ca_certificate_path(),
            checksum_carl_ca_certificate_file: constants::default_checksum_carl_ca_certificate_file(),
            checksum_netbird_ca_certificate_file: constants::default_checksum_netbird_ca_certificate_file(),
            command_runner: Box::new(DefaultCommandRunner),
        }
    }
}

fn write_carl_certificate(new_certificate: Pem, carl_ca_certificate_path: &Path, checksum_carl_ca_certificate_file: &Path) -> anyhow::Result<()> {

    let carl_ca_certificate_dir = carl_ca_certificate_path.parent().unwrap();
    fs::create_dir_all(carl_ca_certificate_dir)
        .context(format!("Unable to create path {:?}", carl_ca_certificate_dir))?;
    
    fs::write(
        carl_ca_certificate_path,
        new_certificate.to_string()
    ).context(format!(
        "Write CA certificate was not successful at location {:?}", carl_ca_certificate_path
    ))?;

    let checksum = util::file_checksum(carl_ca_certificate_path)?;
    let checksum_unpack_file = checksum_carl_ca_certificate_file;
    fs::create_dir_all(checksum_unpack_file.parent().unwrap())?;
    fs::write(checksum_unpack_file, checksum)
        .context(format!("Writing checksum for carl ca certificate to '{}'.", checksum_unpack_file.display()))?;
    
    Ok(())
}

fn write_netbird_certificate(
    carl_ca_certificate_path: &Path, 
    netbird_ca_certificate_path: &Path,
    checksum_netbird_ca_certificate_file: &Path,
    command_runner: &dyn CommandRunner
) -> anyhow::Result<()> {

    let netbird_ca_certificate_dir = netbird_ca_certificate_path.parent().unwrap();
    fs::create_dir_all(netbird_ca_certificate_dir)
        .context(format!("Unable to create path {:?}", netbird_ca_certificate_dir))?;

    fs::copy(
        carl_ca_certificate_path,
        netbird_ca_certificate_path,
    ).context(format!(
        "Copying CA certificate from {:?} to {:?} was not possible.", carl_ca_certificate_path, netbird_ca_certificate_path
    ))?;

    let update_ca_certificates = which::which("update-ca-certificates")
        .context(String::from("No command `update-ca-certificates` found. Ensure your system provides this command."))?;

    command_runner.run(
        &mut Command::new(update_ca_certificates) //Update OS certificate store, as NetBird reads from there
    ).context("update-ca-certificates could not be executed successfully!")?;

    let checksum = util::file_checksum(netbird_ca_certificate_path)?;
    let checksum_unpack_file = checksum_netbird_ca_certificate_file;
    fs::create_dir_all(checksum_unpack_file.parent().unwrap())?;
    fs::write(checksum_unpack_file, checksum)
        .context(format!("Writing checksum for netbird ca certificate to '{}'.", checksum_unpack_file.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use pem::Pem;

    use opendut_types::util::net::Certificate;

    use crate::setup::task::{Task, TaskFulfilled};
    use crate::setup::tasks::WriteCaCertificate;
    use crate::setup::util::NoopCommandRunner;

    #[test]
    fn should_check_task_is_fulfilled() -> anyhow::Result<()> {
        let temp = TempDir::new().unwrap();

        let carl_ca_certificate_path = temp.child("ca.pem");

        let netbird_ca_certificate_path = temp.child("opendut-ca.crt");

        let checksum_carl_ca_certificate_file = temp.child("ca.pem.checksum");

        let checksum_netbird_ca_certificate_file = temp.child("opendut-ca.crt.checksum");

        let task = WriteCaCertificate {
            certificate: Certificate(Pem::new("Test Tag".to_string(), vec![])),
            carl_ca_certificate_path: carl_ca_certificate_path.to_path_buf(),
            netbird_ca_certificate_path: netbird_ca_certificate_path.to_path_buf(),
            checksum_carl_ca_certificate_file: checksum_carl_ca_certificate_file.to_path_buf(),
            checksum_netbird_ca_certificate_file: checksum_netbird_ca_certificate_file.to_path_buf(),
            command_runner: Box::new(NoopCommandRunner),
        };

        assert_eq!(task.check_fulfilled()?, TaskFulfilled::No);
        task.execute()?;
        assert_eq!(task.check_fulfilled()?, TaskFulfilled::Yes);

        Ok(())
    }
}
