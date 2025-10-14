pub mod client;
pub mod config;
pub mod reqwest_client;
pub mod tonic_service;
pub mod error;
pub mod pem;
pub mod middleware;
mod authenticator;
use thiserror::Error;
use url::{Url};

#[derive(Clone, Debug)]
pub struct IssuerUrl(pub Url);
#[derive(Debug, Error)]
pub enum InvalidIssuerUrl {
    #[error("URL parse error: {0}")]
    ParseError(#[from] url::ParseError),
    #[error("Issuer URL does not end with a trailing slash")]
    MissingTrailingSlash(String),
}

impl IssuerUrl {
    pub fn value(&self) -> &Url {
        &self.0
    }
}

impl TryFrom<&str> for IssuerUrl {
    type Error = InvalidIssuerUrl;

    fn try_from(issuer: &str) -> Result<Self, Self::Error> {
        let issuer_url = Url::parse(issuer)
            .map_err(InvalidIssuerUrl::ParseError)?;

        if issuer_url.as_str().ends_with('/') {
            Ok(Self(issuer_url))
        } else {
            Err(InvalidIssuerUrl::MissingTrailingSlash(issuer.to_string()))
        }
    }
}

impl TryFrom<&String> for IssuerUrl {
    type Error = InvalidIssuerUrl;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}