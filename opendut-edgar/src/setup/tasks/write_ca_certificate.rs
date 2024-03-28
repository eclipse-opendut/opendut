use std::fs;
use std::ops::Not;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;
use anyhow::Context;
use pem::Pem;
use serde::{Deserialize, Serialize};
use tracing::debug;
use opendut_types::util::net::Certificate;
use crate::setup::constants;
use crate::setup::task::{Success, Task, TaskFulfilled};
use crate::setup::util::EvaluateRequiringSuccess;


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WriteCaCertificate {
    pub certificate: Certificate,
}

impl Task for WriteCaCertificate {

    fn description(&self) -> String {
        String::from("Write CA Certificates")
    }

    fn check_fulfilled(&self) -> anyhow::Result<TaskFulfilled> {
        if constants::carl_ca_certificate_path().is_file()
        && constants::netbird_ca_certificate_path().is_file() {
            Ok(TaskFulfilled::Yes)
        } else {
            Ok(TaskFulfilled::No)
        }
    }

    fn execute(&self) -> anyhow::Result<Success> {
        let Certificate(new_certificate) = Clone::clone(&self.certificate);

        let carl_ca_certificate_path = constants::carl_ca_certificate_path();

        write_carl_certificate(new_certificate, &carl_ca_certificate_path)?;

        write_netbird_certificate(&carl_ca_certificate_path)?; //TODO this certificate doesn't have to be the same as for CARL and should instead be retrieved from CARL after the initial connection

        Ok(Success::default())
    }
}

fn write_carl_certificate(new_certificate: Pem, carl_ca_certificate_path: &Path) -> anyhow::Result<()> {

    let carl_ca_certificate_dir = carl_ca_certificate_path.parent().unwrap();
    fs::create_dir_all(carl_ca_certificate_dir)
        .context(format!("Unable to create path {:?}", carl_ca_certificate_dir))?;

    // Read ca.pem from disk if it exists and compare the tags and contents of this file with the one provided from the setup string
    let is_ca_cert_file_already_existing_and_equal_to_setup_string_cert =
        if carl_ca_certificate_path.is_file() {
            let existing_ca_cert_file = fs::read_to_string(carl_ca_certificate_path)
                .expect("Could not read the CA certificate file from disk.");

            match Pem::from_str(&existing_ca_cert_file) {
                Ok(file_certificate) => {
                    if file_certificate.contents() == new_certificate.contents()
                    && file_certificate.tag() == new_certificate.tag() {
                        debug!("CA certificate file already existing and tags and contents are equal to each other. Only executing command update-ca-certificates.");
                        true
                    } else {
                        false
                    }
                }
                Err(_) => { false }
            }
        } else {
            false
        };

    // Write ca.pem from setup string to disk, only if check fails
    if is_ca_cert_file_already_existing_and_equal_to_setup_string_cert.not() {
        fs::write(
            carl_ca_certificate_path,
            new_certificate.to_string()
        ).context(format!(
            "Write CA certificate was not successful at location {:?}", carl_ca_certificate_path
        ))?;
    }

    Ok(())
}

fn write_netbird_certificate(carl_ca_certificate_path: &Path) -> anyhow::Result<()> {
    let netbird_ca_certificate_path = constants::netbird_ca_certificate_path();

    let netbird_ca_certificate_dir = netbird_ca_certificate_path.parent().unwrap();
    fs::create_dir_all(netbird_ca_certificate_dir)
        .context(format!("Unable to create path {:?}", netbird_ca_certificate_dir))?;

    fs::copy(
        carl_ca_certificate_path,
        constants::netbird_ca_certificate_path(),
    ).context(format!(
        "Copying CA certificate from {:?} to {:?} was not possible.", carl_ca_certificate_path, constants::netbird_ca_certificate_path()
    ))?;

    Command::new("update-ca-certificates") //Update OS certificate store, as NetBird reads from there
        .evaluate_requiring_success()
        .context("update-ca-certificates could not be executed successfully!")?;

    Ok(())
}
