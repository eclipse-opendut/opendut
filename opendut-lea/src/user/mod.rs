use leptos::context::provide_context;
use leptos::prelude::{Effect, Get, RwSignal, Set};
use leptos_oidc::{Algorithm, Auth, AuthSignal, TokenData};
use leptos_router::hooks::use_navigate;
use opendut_auth::types::Claims;
pub use overview::UserOverview;
use crate::routing::{navigate_to, WellKnownRoutes};
use leptos_oidc::AuthenticatedData as LeptosOidcAuthenticatedData;
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

pub type UserAuthenticationSignal = RwSignal<UserAuthentication>;

#[derive(Debug, Clone, Default)]
pub enum UserAuthentication {
    #[default]
    Loading,
    Disabled,
    Unauthenticated,
    Authenticated(Box<AuthenticatedData>),
}

#[derive(Debug, Clone)]
pub struct AuthenticatedData {
    auth: LeptosOidcAuthenticatedData,
    token: Option<TokenData<Claims>>,
}

impl UserAuthentication {
    pub fn _user(&self) -> Option<TokenData<Claims>> {
        match self {
            UserAuthentication::Authenticated(data) => {
                data.token.clone()
            }
            _ => { None }
        }
    }

    pub fn _has_group(&self, group: &str) -> Option<bool> {
        
        match self {
            UserAuthentication::Loading => { None }
            UserAuthentication::Disabled=> { Some(true) }
            UserAuthentication::Unauthenticated=> { Some(false) }
            UserAuthentication::Authenticated(data) => {
                match data.token.as_ref() {
                    None => { Some(false) }
                    Some(user) => { 
                        Some(user.claims.additional_claims.has_group(group)) 
                    }
                }
            }
        }
    }
    
    pub fn is_authenticated(&self) -> Option<bool> {
        match self {
            UserAuthentication::Loading => { None }
            UserAuthentication::Disabled=> { Some(true) }
            UserAuthentication::Unauthenticated=> { Some(false) }
            UserAuthentication::Authenticated(data) => {
                Some(data.auth.is_authenticated())
            }
        }
    }

    pub fn username(&self) -> String {
        let name = match self {
            UserAuthentication::Loading => { "loading" }
            UserAuthentication::Disabled=> { "disabled" }
            UserAuthentication::Unauthenticated=> { "unauthenticated" }
            UserAuthentication::Authenticated(data) => {
                match data.token.as_ref() {
                    None => { UNAUTHENTICATED_USER }
                    Some(user) => { &user.claims.preferred_username }
                }
            }
        };
        name.to_string()
    }

    pub fn fullname(&self) -> Option<String> {
        match self {
            UserAuthentication::Authenticated(data) => {
                if let Some(user) = data.token.as_ref() {
                    let name = &user.claims.name;
                    Some(name.to_string())
                } else {
                    Some(UNAUTHENTICATED_USER.to_string())
                }
            }
            _ => None
        }
    }

    pub fn  email(&self) -> Option<String> {
        match self {
            UserAuthentication::Authenticated(data) => {
                if let Some(user) = data.token.as_ref() {
                    let email = &user.claims.email;
                    Some(email.to_string())
                } else {
                    Some(UNAUTHENTICATED_USER.to_string())
                }
            }
            _ => None
        }
    }
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
        tracing::debug!("Running auth effect switch: {auth_config_switch:?} auth: {auth:?}");

        match auth_config_switch {
            AuthenticationConfigSwitch::Loading => {
                user_auth.set(UserAuthentication::default());
            }
            AuthenticationConfigSwitch::Disabled => {
                tracing::debug!("user disabled");
                user_auth.set(UserAuthentication::Disabled);
            }
            AuthenticationConfigSwitch::Enabled => {
                match auth {
                    Auth::Loading => {
                        tracing::trace!("user still loading");
                        user_auth.set(UserAuthentication::Loading);
                    }
                    Auth::Unauthenticated(_) => {
                        tracing::trace!("user loaded, unauthenticated");
                        user_auth.set(UserAuthentication::Unauthenticated);
                    }
                    Auth::Authenticated(auth) => {
                        tracing::debug!("user authenticated");
                        let token = auth.decoded_access_token::<Claims>(Algorithm::RS256, &[DEFAULT_TOKEN_AUDIENCE]);
                        user_auth.set(UserAuthentication::Authenticated(Box::new(AuthenticatedData { auth, token })));
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
