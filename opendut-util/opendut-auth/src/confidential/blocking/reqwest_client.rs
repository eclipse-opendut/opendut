use anyhow::anyhow;
use config::Config;
use oauth2::{HttpRequest, HttpResponse};
use pem::Pem;
use reqwest::blocking::Client;
use reqwest::Certificate;

use crate::confidential::error::OidcClientError;
use crate::confidential::pem::PemFromConfig;

#[derive(Debug, Clone)]
pub struct OidcBlockingReqwestClient {
    pub(crate) client: Client,
}

const CONFIG_KEY_GENERIC_CA: &str = "network.tls.ca";
const CONFIG_KEY_OIDC_CA: &str = "network.oidc.client.ca";

impl OidcBlockingReqwestClient {
    pub async fn from_config(config: &Config) -> anyhow::Result<Self> {
        match Pem::from_config_path(CONFIG_KEY_OIDC_CA, config).await {
            Ok(ca_certificate) => {
                let client = OidcBlockingReqwestClient::build_client(ca_certificate)?;
                Ok(Self { client })
            }
            Err(_error) => {
                // could not find specific OIDC CA, try generic CA
                match Pem::from_config_path(CONFIG_KEY_GENERIC_CA, config).await {
                    Ok(ca_certificate) => {
                        Ok(Self { client: OidcBlockingReqwestClient::build_client(ca_certificate)? })
                    }
                    Err(error) => {
                        Err(anyhow!("Could not find any CA certificate in config. Error: {}", error))
                    }
                }
            }
        }
    }

    fn build_client(ca_certificate: Pem) -> anyhow::Result<Client> {
        let reqwest_certificate = Certificate::from_pem(ca_certificate.to_string().as_bytes().iter().as_slice())
            .map_err(|cause| OidcClientError::LoadCustomCA(cause.to_string()))?;
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .add_root_certificate(reqwest_certificate)
            .build()
            .map_err(|cause| OidcClientError::LoadCustomCA(cause.to_string()))?;
        Ok(client)
    }

    pub fn from_pem(ca_certificate: Pem) -> anyhow::Result<Self> {
        Ok(Self { client: OidcBlockingReqwestClient::build_client(ca_certificate)? })
    }

    pub fn client(&self) -> Client {
        self.client.clone()
    }

    pub fn sync_http_client(
        &self,
        request: HttpRequest,
    ) -> Result<HttpResponse, OidcClientError> {
        let client = self.client.clone();
        let mut request_builder = client
            .request(request.method, request.url.as_str())
            .body(request.body);
        for (name, value) in &request.headers {
            request_builder = request_builder.header(name.as_str(), value.as_bytes());
        }
        let request = request_builder.build()
            .map_err(|cause| {
                OidcClientError::AuthReqwest { message: cause.to_string(), status: cause.status().unwrap_or_default().to_string(), inner: cause }
            })?;
        let response = client.execute(request)
            .map_err(|cause: reqwest::Error| {
                OidcClientError::AuthReqwest { message: cause.to_string(), status: cause.status().unwrap_or_default().to_string(), inner: cause }
            })?;
        let status_code = response.status();
        let headers = response.headers().to_owned();
        let data = response.bytes()
            .map_err(|cause| {
                OidcClientError::AuthReqwest { message: cause.to_string(), status: cause.status().unwrap_or_default().to_string(), inner: cause }
            })?;
        Ok(HttpResponse {
            status_code,
            headers,
            body: data.to_vec(),
        })
    }
}
