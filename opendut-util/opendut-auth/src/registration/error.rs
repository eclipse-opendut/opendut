use std::fmt::Display;

use openidconnect::core::CoreRegisterErrorResponseType;
use openidconnect::registration::ClientRegistrationError;

use crate::confidential::error::OidcClientError;

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
            format!("RegistrationClientError Other: {error:?}")
        }
        ClientRegistrationError::Parse(error) => {
            format!("RegistrationClientError Parse: {error:?}")
        }
        ClientRegistrationError::Request(error) => {
            format!("RegistrationClientError Request: {error:?}")
        }
        ClientRegistrationError::Response(status, _body,  error) => {
            format!("RegistrationClientError Response: {error} Status: {status}")
        }
        ClientRegistrationError::Serialize(error) => {
            format!("RegistrationClientError Serialize: {error:?}")
        }
        ClientRegistrationError::ServerResponse(error) => {
            format!("RegistrationClientError ServerResponse: {error:?}")
        }
        _ => {
            format!("RegistrationClientError Phantom: {error:?}")
        }
    }
}

