use std::fmt::{Debug, Formatter};

use reqwest::header::HeaderValue;
use reqwest::header::InvalidHeaderValue;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct NetbirdToken {
    pub(crate) r#type: TokenType,
    pub(crate) value: String
}
#[derive(Clone, Deserialize)]
pub enum TokenType {
    PersonalAccess,
    Bearer,
}

impl NetbirdToken {
    pub fn new_personal_access(value: impl Into<String>) -> Self {
        Self {
            r#type: TokenType::PersonalAccess,
            value: value.into(),
        }
    }
    pub fn new_bearer(value: impl Into<String>) -> Self {
        Self {
            r#type: TokenType::Bearer,
            value: value.into(),
        }
    }

    pub fn sensitive_header(&self) -> Result<HeaderValue, InvalidHeaderValue> {
        let header = match self.r#type {
            TokenType::PersonalAccess => format!("Token {}", self.value),
            TokenType::Bearer => format!("Bearer {}", self.value),
        };
        let mut header = HeaderValue::from_str(&header)?;
        header.set_sensitive(true);
        Ok(header)
    }
}

impl Debug for NetbirdToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.r#type {
            TokenType::PersonalAccess => write!(f, "Token::PersonalAccess(****)"),
            TokenType::Bearer => write!(f, "Token::Bearer(****)"),
        }
    }
}
