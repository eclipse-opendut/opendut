use std::fmt::Display;

use oauth2::basic::BasicErrorResponse;
use oauth2::RequestTokenError;
use openidconnect::core::CoreRegisterErrorResponseType;
use openidconnect::registration::ClientRegistrationError;
use reqwest::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OidcClientError {
    #[error("AuthReqwest Error, request failed: '{message}' status: '{status}' inner: '{inner}'")]
    AuthReqwest {
        message: String,
        status: String,
        inner: Error,
    },
    #[error("Failed to load custom certificate authority: {}", _0)]
    LoadCustomCA(String),
    #[error("Other error: {}", _0)]
    Other(String),
}

#[derive(thiserror::Error, Debug)]
pub enum ConfidentialClientError {
    #[error("Failed to load OIDC configuration: '{message}'. Cause: '{cause}'")]
    Configuration { message: String, cause: Box<dyn std::error::Error + Send + Sync> },
    #[error("OIDC configuration error: '{message}'.")]
    Other { message: String },
}

/// Printable version of the RequestTokenError with complete error message
#[derive(thiserror::Error, Debug)]
pub struct WrappedRequestTokenError(pub RequestTokenError<OidcClientError, BasicErrorResponse>);
impl Display for WrappedRequestTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", parse_oauth_request_error(&self.0))
    }
}


/// Printable version of the ClientRegistrationError with complete error message
#[derive(thiserror::Error, Debug)]
pub struct WrappedClientRegistrationError(pub ClientRegistrationError<CoreRegisterErrorResponseType, OidcClientError>);
impl Display for WrappedClientRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", parse_client_registration_error(&self.0))
    }
}

fn parse_client_registration_error(error: &ClientRegistrationError<CoreRegisterErrorResponseType, OidcClientError>) -> String {
    match error {
        ClientRegistrationError::Other(error) => {
            format!("RegistrationClientError Other: {:?}", error)
        }
        ClientRegistrationError::Parse(error) => {
            format!("RegistrationClientError Parse: {:?}", error)
        }
        ClientRegistrationError::Request(error) => {
            format!("RegistrationClientError Request: {:?}", error)
        }
        ClientRegistrationError::Response(status, _body,  error) => {
            format!("RegistrationClientError Response: {} Status: {}", error, status)
        }
        ClientRegistrationError::Serialize(error) => {
            format!("RegistrationClientError Serialize: {:?}", error)
        }
        ClientRegistrationError::ServerResponse(error) => {
            format!("RegistrationClientError ServerResponse: {:?}", error)
        }
        _ => {
            format!("RegistrationClientError Phantom: {:?}", error)
        }
    }
}


fn parse_oauth_request_error(error: &RequestTokenError<OidcClientError, BasicErrorResponse>) -> String {
    match error {
        RequestTokenError::ServerResponse(ref server_error) => {
            server_error.error().to_string()
        }
        RequestTokenError::Request(ref request_error) => {
            request_error.to_string()
        }
        RequestTokenError::Parse(ref error_token, ref _error_response) => {
            error_token.to_string()
        }
        RequestTokenError::Other(ref other) => {
            other.to_string()
        }
    }
}
