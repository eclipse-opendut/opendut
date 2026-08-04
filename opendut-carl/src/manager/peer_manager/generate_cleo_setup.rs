use opendut_auth::registration::client::RegistrationClientRef;
use opendut_auth::registration::resources::UserId;
use opendut_auth::types::SCOPE_ADMIN_API;
use opendut_model::cleo::{CleoId, CleoSetup};
use opendut_model::util::net::{AuthConfig, Certificate};
use tracing::debug;
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

            // Assign admin API scope to the CLEO client
            let keycloak_client_uuid = registration_client.find_client_uuid(&client_credentials.client_id.clone().value())
                .await
                .map_err(|cause| GenerateCleoSetupError::Internal { cause: cause.to_string() })?;
            registration_client.assign_scope_to_client(&keycloak_client_uuid, SCOPE_ADMIN_API)
                .await
                .map_err(|cause| GenerateCleoSetupError::Internal { cause: cause.to_string() })?;
            debug!("Assigned scope '{SCOPE_ADMIN_API}' to CLEO client <{cleo_id}>.");

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
    use crate::manager::testing::get_cert;
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

}
