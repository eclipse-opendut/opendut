use std::str::FromStr;
use anyhow::anyhow;
use config::Config;
use oauth2::{HttpRequest, HttpResponse};
use pem::Pem;
use reqwest::Certificate;

use crate::confidential::error::OidcClientError;
use crate::confidential::pem::PemFromConfig;

#[derive(Debug, Clone)]
pub struct OidcReqwestClient {
    pub(crate) client: reqwest::Client,
}

const CONFIG_KEY_GENERIC_CA_CONTENT: &str = "network.tls.ca.content";
const CONFIG_KEY_GENERIC_CA: &str = "network.tls.ca";
const CONFIG_KEY_OIDC_CA: &str = "network.oidc.client.ca";

impl OidcReqwestClient {
    pub async fn from_config(config: &Config) -> anyhow::Result<Self> {
        match Pem::from_config_path(CONFIG_KEY_OIDC_CA, config).await {
            Ok(ca_certificate) => {
                let client = OidcReqwestClient::build_client(ca_certificate)?;
                Ok(Self { client })
            }
            Err(_error) => {
                // could not find specific OIDC CA, try generic CA
                match config.get_string(CONFIG_KEY_GENERIC_CA_CONTENT) {
                    Ok(ca_content) => {
                        let ca_certificate = Pem::from_str(&ca_content)
                            .map_err(|error| OidcClientError::LoadCustomCA(format!("Could not parse CA from configuration. Error: {error}")))?;
                        Ok(Self { client: OidcReqwestClient::build_client(ca_certificate)? })
                    }
                    Err(_) => {
                        match Pem::from_config_path(CONFIG_KEY_GENERIC_CA, config).await {
                            Ok(ca_certificate) => {
                                Ok(Self { client: OidcReqwestClient::build_client(ca_certificate)? })
                            }
                            Err(error) => {
                                Err(anyhow!("Could not find any CA certificate in config. Error: {}", error))
                            }
                        }
                    }
                }
            }
        }
    }

    fn build_client(ca_certificate: Pem) -> anyhow::Result<reqwest::Client> {
        let reqwest_certificate = Certificate::from_pem(ca_certificate.to_string().as_bytes().iter().as_slice())
            .map_err(|cause| OidcClientError::LoadCustomCA(cause.to_string()))?;
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .add_root_certificate(reqwest_certificate)
            .build()
            .map_err(|cause| OidcClientError::LoadCustomCA(cause.to_string()))?;
        Ok(client)
    }

    pub fn from_pem(ca_certificate: Pem) -> anyhow::Result<Self> {
        Ok(Self { client: OidcReqwestClient::build_client(ca_certificate)? })
    }

    pub fn client(&self) -> reqwest::Client {
        self.client.clone()
    }

    pub async fn async_http_client(
        &self,
        request: HttpRequest,
    ) -> Result<HttpResponse, OidcClientError> {
        let client = self.client.clone();
        let mut request_builder = client
            .request(request.method().clone(), request.uri().to_string())
            .body(request.body().clone());
        for (name, value) in request.headers() {
            request_builder = request_builder.header(name.as_str(), value.as_bytes());
        }
        let request = request_builder.build()
            .map_err(|cause| {
                OidcClientError::AuthReqwest { message: cause.to_string(), status: cause.status().unwrap_or_default().to_string(), inner: cause }
            })?;
        let response = client.execute(request).await
            .map_err(|cause: reqwest::Error| {
                OidcClientError::AuthReqwest { message: cause.to_string(), status: cause.status().unwrap_or_default().to_string(), inner: cause }
            })?;
        let status_code = response.status();
        let headers = response.headers().to_owned();
        let data = response.bytes().await
            .map_err(|cause| {
                OidcClientError::AuthReqwest { message: cause.to_string(), status: cause.status().unwrap_or_default().to_string(), inner: cause }
            })?;

        let returned_response = {
            let mut returned_response = http::Response::builder()
                .status(status_code);
            for (name, value) in headers.iter() {
                returned_response = returned_response.header(name, value);
            }
            returned_response
                .body(data.to_vec())
                .map_err(|cause| {
                    OidcClientError::Other(format!("Failed to build response body: {cause}"))
                })?
        };

        Ok(returned_response)
    }
}
