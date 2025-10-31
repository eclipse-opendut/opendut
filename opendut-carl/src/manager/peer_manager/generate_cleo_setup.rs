use opendut_auth::registration::client::RegistrationClientRef;
use opendut_auth::registration::resources::UserId;
use opendut_model::cleo::{CleoId, CleoSetup};
use opendut_model::util::net::{AuthConfig, Certificate};
use tracing::{debug, error};
use url::Url;
use opendut_util::pem::Pem;

pub struct GenerateCleoSetupParams {
    pub cleo: CleoId,
    pub carl_url: Url,
    pub ca: Pem,
    pub oidc_registration_client: Option<RegistrationClientRef>,
    pub user_id: UserId,
}

#[tracing::instrument(skip_all, level="trace")]
pub async fn generate_cleo_setup(params: GenerateCleoSetupParams) -> Result<CleoSetup, GenerateCleoSetupError> {

    let cleo_id = params.cleo;
    debug!("Generating CLEO Setup.");

    let auth_config = match params.oidc_registration_client {
        None => {
            AuthConfig::Disabled
        }
        Some(registration_client) => {
            let resource_id = cleo_id.into();
            debug!("Generating OIDC client for CLEO: <{cleo_id}>.");
            let issuer_url = registration_client.config.issuer_remote_url.value().clone();
            let client_credentials = registration_client.register_new_client_for_user(resource_id, params.user_id)
                .await
                .map_err(|cause| GenerateCleoSetupError::Internal { cause: cause.to_string() })?;
            debug!("Successfully generated CLEO setup with id <{cleo_id}>. OIDC client_id='{}'.", client_credentials.client_id.clone().value());
            AuthConfig::from_credentials(issuer_url, client_credentials)
        }
    };

    Ok(CleoSetup {
        id: cleo_id,
        carl: params.carl_url,
        ca: Certificate(params.ca),
        auth_config,
    })
}

#[derive(thiserror::Error, Debug)]
pub enum GenerateCleoSetupError {
    #[error("An internal error occurred while creating a CleoSetup:\n  {cause}")]
    Internal {
        cause: String,
    }
}

#[cfg(test)]
mod tests {
    use googletest::prelude::*;
    use std::str::FromStr;

    use super::*;

    #[tokio::test]
    async fn should_create_setup_string_cleo() -> anyhow::Result<()> {
        let generate_cleo_setup_params = GenerateCleoSetupParams {
            cleo: CleoId::try_from("787d0b11-51f3-4cfe-8131-c7d89d53f0e9")?,
            carl_url: Url::parse("https://example.com:1234").unwrap(),
            ca: get_cert(),
            oidc_registration_client: None,
            user_id: UserId { value: String::from("testUser") },
        };

        let cleo_setup = generate_cleo_setup(generate_cleo_setup_params).await?;
        assert_that!(cleo_setup.id, eq(CleoId::try_from("787d0b11-51f3-4cfe-8131-c7d89d53f0e9")?));
        assert_that!(cleo_setup.auth_config, eq(&AuthConfig::Disabled));
        assert_that!(cleo_setup.carl, eq(&Url::parse("https://example.com:1234")?));

        Ok(())
    }

    pub fn get_cert() -> Pem {
        match Pem::from_str(CERTIFICATE_AUTHORITY_STRING) {
            Ok(cert) => { cert }
            Err(_) => { panic!("Not a valid certificate!") }
        }
    }

    const CERTIFICATE_AUTHORITY_STRING: &str = include_str!("../../../../resources/development/tls/insecure-development-ca.pem");
}
