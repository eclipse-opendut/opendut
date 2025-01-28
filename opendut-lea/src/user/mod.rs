use leptos::context::provide_context;
use leptos::prelude::{Effect, Get, RwSignal, Set};
use leptos_oidc::{Algorithm, Auth, AuthSignal, TokenData};
use leptos_router::hooks::use_navigate;
use serde::{Deserialize, Serialize};
pub use overview::UserOverview;
use crate::routing::{navigate_to, WellKnownRoutes};

/// In case authentication is disabled the user identity is not known
pub const UNAUTHENTICATED_USER: &str = "unknown-user";
mod overview;
const DEFAULT_TOKEN_AUDIENCE: &str = "account";

#[derive(Debug, Clone, Default)]
pub enum AuthenticationConfigSwitch {
    #[default]
    Loading,
    Disabled,
    Enabled,
}

#[derive(Debug, Clone, Default)]
pub enum UserAuthentication {
    #[default]
    Loading,
    Disabled,
    Unauthenticated,
    Authenticated(Option<Box<TokenData<Claims>>>),
}


pub(crate) fn provide_authentication_signals_in_context() -> AuthSignal {
    let auth = Auth::signal();
    provide_context(auth);
    let user_auth = RwSignal::new(UserAuthentication::default());
    provide_context(user_auth);
    let auth_config_switch = RwSignal::new(AuthenticationConfigSwitch::Loading);
    provide_context(auth_config_switch);

    Effect::new(move || {
        let auth = auth.get();
        let auth_config_switch = auth_config_switch.get();
        tracing::debug!("Running auth effect: {auth:?}");

        match auth_config_switch {
            AuthenticationConfigSwitch::Loading => {
                tracing::debug!("user loading");
                user_auth.set(UserAuthentication::default());
            }
            AuthenticationConfigSwitch::Disabled => {
                tracing::debug!("user disabled");
                user_auth.set(UserAuthentication::Disabled);
            }
            AuthenticationConfigSwitch::Enabled => {
                match auth {
                    Auth::Loading => {
                        tracing::debug!("user still loading");
                        user_auth.set(UserAuthentication::Loading);
                    }
                    Auth::Unauthenticated(_) => {
                        tracing::debug!("user loaded, unauthenticated");
                        user_auth.set(UserAuthentication::Unauthenticated);
                    }
                    Auth::Authenticated(auth) => {
                        tracing::debug!("user authenticated");
                        let token = auth.decoded_access_token::<Claims>(Algorithm::RS256, &[DEFAULT_TOKEN_AUDIENCE])
                            .map(|token| Box::new(token));
                        user_auth.set(UserAuthentication::Authenticated(token));
                    }
                    Auth::Error(error) => {
                        user_auth.set(UserAuthentication::Unauthenticated);
                        let error_message = format!("Error while initializing the authentication stack: {error}");
                        tracing::error!(error_message);
                
                        navigate_to(
                            WellKnownRoutes::ErrorPage {
                                title: String::from("Initialization error"),
                                text: error_message,
                                details: None,
                            },
                            use_navigate()
                        );
                    }
                }
            
                
            }

        }
    });
    
    auth
}

// TODO: move to opendut-types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Claims {
    /// Audience
    #[serde(rename = "aud")]
    audience: String,
    /// Issued at (as UTC timestamp)
    #[serde(rename = "iat")]
    issued_at: usize,
    /// Issuer
    #[serde(rename = "iss")]
    issuer: String,
    /// Expiration time (as UTC timestamp)
    #[serde(rename = "exp")]
    expiration_utc: usize,
    /// Subject (whom token refers to)
    #[serde(rename = "sub")]
    subject: String,
    // Roles the user belongs to (custom claim if present)
    #[serde(default = "Claims::empty_vector")]
    roles: Vec<String>,
    // Groups of the user (custom claim if present)
    #[serde(default = "Claims::empty_vector")]
    groups: Vec<String>,
    // Name of the user
    name: String,
    // Email address of the user
    email: String,
    // Username of the user
    pub preferred_username: String,
}

impl Claims {
    pub(crate) fn empty_vector() -> Vec<String> { Vec::new() }
}