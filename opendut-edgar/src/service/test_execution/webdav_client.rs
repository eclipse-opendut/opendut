use config::Config;
use pem::Pem;
use reqwest::{Method, RequestBuilder, Url, Body, Response, header, Certificate};
use tracing::error;

use opendut_auth::confidential::pem::PemFromConfig;

pub struct WebdavClient {
    bearer_token: String,
    client: reqwest::Client,
}

const CONFIG_KEY_GENERIC_CA: &str = "network.tls.ca";

impl WebdavClient {

    pub async fn new(bearer_token: String, config: &Config) -> Result<Self, Error> {
        Ok(Self { 
            bearer_token,
            client: WebdavClient::build_client(config).await?,
        })
    }

    async fn build_client(config: &Config) -> Result<reqwest::Client, Error> {
        let ca_certificate = Pem::from_config_path(CONFIG_KEY_GENERIC_CA, config).await
            .map_err(|cause| Error::LoadCustomCA { message: format!("Could not find any CA certificate in config: {cause}") })?;

        let reqwest_certificate = Certificate::from_pem(ca_certificate.to_string().as_bytes().iter().as_slice())
            .map_err(|cause| Error::LoadCustomCA { message: cause.to_string() } )?;
        let client = reqwest::Client::builder()
            .add_root_certificate(reqwest_certificate)
            .build()
            .map_err(|cause| Error::LoadCustomCA { message: cause.to_string() } )?;

        Ok(client)
    }

    fn start_request(&self, method: Method, path: Url) -> RequestBuilder {
        self.client
            .request(method, path)
            .bearer_auth(self.bearer_token.clone())
    }

    fn custom_header(&self, name: &str, value: &str) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::HeaderName::from_bytes(name.as_bytes()).unwrap(), header::HeaderValue::from_bytes(value.as_bytes()).unwrap());
        headers
    }

    /// Upload a file/zip on Webdav server
    ///
    /// It can be any type of file as long as it is transformed to a vector of bytes (Vec<u8>).
    /// This can be achieved with **std::fs::File** or **zip-rs** for sending zip files.
    ///
    /// Use absolute path to the webdav server folder location
    pub async fn put<B: Into<Body>>(&self, body: B, path: Url) -> Result<Response, Error> {
        self.start_request(Method::PUT, path)
            .headers(self.custom_header("content-type", "application/octet-stream"))
            .body(body)
            .send()
            .await
            .map_err(|cause| Error::Request { method: String::from("PUT"), cause } )
    }

    pub async fn mkcol(&self, path: Url) -> Result<Response, Error> {
        self.start_request(Method::from_bytes(b"MKCOL").unwrap(), path)
            .send()
            .await
            .map_err(|cause| Error::Request { method: String::from("MKCOL"), cause } )
    }

    pub async fn create_collection_path(&self, path: Url) -> Result<(), Error>{
        let mut accumulated_path = String::from("/");
        let path_segments = path.path_segments()
            .ok_or(Error::Other { message: format!("URL cannot be split into path segments: {path}") })?
            .filter(|p| !p.is_empty());

        for segment in path_segments{
            accumulated_path.push_str(segment);
            accumulated_path.push('/');
            
            // The '/' in the beginning of the accumulated path causes the existing path in the URL to be dropped
            let partial_url = path.join(&accumulated_path)
                .map_err(|cause| Error::Other { message: format!("Failed to join partial path '{accumulated_path}' to base URL: {cause}") } )?;

            let response = self.mkcol(partial_url.clone())
                .await?;

            match response.status().as_u16() {
                201 | 405 => (),
                _ => error!("Unexpected response code while trying to create collection {partial_url}"),
            }
        }

        Ok(())
    }

}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failure while sending WebDAV '{method}' request: {cause}")]
    Request { method: String, cause: reqwest::Error },
    #[error("Failed to load custom certificate authority: {message}")]
    LoadCustomCA { message: String },
    #[error("{message}")]
    Other { message: String },
}