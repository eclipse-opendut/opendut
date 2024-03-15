use std::fs;
use std::ops::Not;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use anyhow::Context;
use pem::Pem;
use serde::{Deserialize, Serialize};
use tracing::debug;
use opendut_types::util::net::Certificate;
use crate::setup::constants::{CA_CERTIFICATE_FILE_NAME_CA_PEM, CA_CERTIFICATE_FILE_NAME_OPENDUT_CA_CRT, CA_CERTIFICATE_LOCATION, SETUP_STRING_CERTIFICATE_STORE_LOCATION};
use crate::setup::task::{Success, Task, TaskFulfilled};
use crate::setup::tasks::create_ca_authority::CaCertificateStoreLocations::{CaCertificateLocation, CaCertificateLocationEmpty, SetupStringCertificateStoreLocation, SetupStringCertificateStoreLocationEmpty};
use crate::setup::util::EvaluateRequiringSuccess;

#[derive(Debug)]
enum CaCertificateStoreLocations {
    SetupStringCertificateStoreLocationEmpty,
    SetupStringCertificateStoreLocation,
    CaCertificateLocationEmpty,
    CaCertificateLocation,
}

fn get_location(cert_location: CaCertificateStoreLocations) -> String {
    match cert_location {
        SetupStringCertificateStoreLocationEmpty => {
            SETUP_STRING_CERTIFICATE_STORE_LOCATION.to_string()
        }
        SetupStringCertificateStoreLocation => {
            format!("{}{}", SETUP_STRING_CERTIFICATE_STORE_LOCATION, CA_CERTIFICATE_FILE_NAME_CA_PEM)
        }
        CaCertificateLocationEmpty => {
            CA_CERTIFICATE_LOCATION.to_string()
        }
        CaCertificateLocation => {
            format!("{}{}", CA_CERTIFICATE_LOCATION, CA_CERTIFICATE_FILE_NAME_OPENDUT_CA_CRT)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CaCertificate {
    pub certificate: Certificate,
}

impl Task for CaCertificate {

    fn description(&self) -> String {
        "Handle CA Certificate from setup string.".to_string()
    }

    fn check_fulfilled(&self) -> anyhow::Result<TaskFulfilled> {
        if PathBuf::from(get_location(CaCertificateLocationEmpty)).is_dir()
            && PathBuf::from(get_location(CaCertificateLocation)).is_file() {
            Ok(TaskFulfilled::Yes)
        } else {
            Ok(TaskFulfilled::No)
        }
    }

    fn execute(&self) -> anyhow::Result<Success> {
        let ca_cert_file = Clone::clone(self);

        // Create '/etc/opendut/tls/' directory if it is not yet existing
        fs::create_dir_all(get_location(SetupStringCertificateStoreLocationEmpty))
            .expect(
                format!("Unable to create path {:?}",
                        get_location(SetupStringCertificateStoreLocationEmpty)
                ).as_str()
            );

        // Read ca.pem from disk if it exists and compare the tags and contents of this file with the one provided from the setup string
        let mut is_ca_cert_file_already_existing_and_equal_to_setup_string_cert = false;

        if PathBuf::from(get_location(SetupStringCertificateStoreLocation)).is_file() {
            let existing_ca_cert_file = fs::read_to_string(get_location(SetupStringCertificateStoreLocation))
                .expect("Could not read the CA certificate file from disk.");

            is_ca_cert_file_already_existing_and_equal_to_setup_string_cert = match Pem::from_str(existing_ca_cert_file.as_str()) {
                Ok(file) => {
                    if file.contents() == ca_cert_file.certificate.0.contents()
                    && file.tag() == ca_cert_file.certificate.0.tag() {
                        debug!("CA certificate file already existing and tags and contents are equal to each other. Only executing command update-ca-certificates.");
                        true
                    } else {
                        false
                    }
                }
                Err(_) => { false }
            };
        }

        // Write ca.pem from setup string to disk, only if check fails
        if is_ca_cert_file_already_existing_and_equal_to_setup_string_cert.not() {
            fs::write(
                get_location(SetupStringCertificateStoreLocation),
                ca_cert_file.certificate.0.to_string()
            ).expect(
                format!("Write CA certificate was not successful at location {:?}",
                        get_location(SetupStringCertificateStoreLocationEmpty)
                ).as_str()
            );
        }

        // Copy ca.pem and execute command 'update-ca-certificates'
        fs::copy(
            get_location(SetupStringCertificateStoreLocation),
            get_location(CaCertificateLocation)
        ).expect(
            format!("Copying CA certificate from {:?} to {:?} was not possible.",
                    get_location(SetupStringCertificateStoreLocationEmpty),
                    get_location(CaCertificateLocationEmpty)
            ).as_str()
        );

        Command::new("update-ca-certificates")
            .evaluate_requiring_success()
            .context("update-ca-certificates could not be executed successfully!")?;

        Ok(Success::default())
    }
}
