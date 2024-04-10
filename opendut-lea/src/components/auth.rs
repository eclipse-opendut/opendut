use leptos::{ChildrenFn, component, create_effect, IntoView, ReadSignal, Show, SignalSet, Transition, use_context, view, ViewFn, WriteSignal};
use leptos_oidc::{Algorithm, DecodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};
use opendut_carl_api::carl::wasm::{AuthData, OptionalAuthData};

use crate::app::{ExpectGlobals, use_app_globals};

#[must_use]
#[component(transparent)]
pub fn LeaAuthenticated(
    children: ChildrenFn,
    #[prop(optional, into)] loading: ViewFn,
    #[prop(optional, into)] unauthenticated: ViewFn,
    #[prop(optional, into)] disabled_auth: ViewFn,
) -> impl IntoView {
    let auth = use_app_globals().expect_auth();
    let app_config = use_app_globals().expect_config();

    match (app_config.idp_config, auth) {
        (Some(lea_idp_config), Some(auth)) => {
            let auth_cloned = auth.clone();
            let auth_token = move || auth_cloned.access_token();
            create_effect(move |_| {
                let (_auth_data, auth_data_write) = use_context::<(ReadSignal<OptionalAuthData>, WriteSignal<OptionalAuthData>)>().expect("AuthData should be provided in the context.");
                if let Some(token) = auth_token() {
                    tracing::debug!("AUTH Token: {}", token);
                    let data = decode_token(&token, lea_idp_config.issuer_url.as_ref());
                    auth_data_write.set(OptionalAuthData {
                        auth_data: Some(
                            AuthData {
                                access_token: token.clone(),
                                preferred_username: data.claims.preferred_username.clone(),
                                name: data.claims.name.clone(),
                                email: data.claims.email.clone(),
                                groups: data.claims.groups.clone(),
                                roles: data.claims.roles.clone(),
                            }
                        )
                    });
                    token
                } else {
                    tracing::debug!("NO TOKEN");
                    "no token".to_string()
                }
            });
            let unauthenticated = move || unauthenticated.run();
            let authenticated = move || auth.authenticated();

            view! {
                <Transition fallback=loading>
                    <Show
                        when=authenticated.clone()
                        fallback=unauthenticated.clone()
                        children=children.clone()
                    />
                </Transition>
            }

        }
        (Some(_lea_idp_config), None) => {
            tracing::warn!("Warning: Authentication enabled - User not authenticated.");
            disabled_auth.run()
        }
        (None, Some(_auth)) => {
            tracing::warn!("Warning: Authentication disabled - No authentication config provided.");
            disabled_auth.run()
        }
        _ => {
            tracing::warn!("Warning: Authentication disabled - Neither an authentication config provided, nor is the user authenticated.");
            disabled_auth.run()
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
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
    preferred_username: String,
}

impl Claims {
    pub(crate) fn empty_vector() -> Vec<String> { Vec::new() }
}

pub(crate) fn decode_token(token: &str, issuer_url: &str) -> TokenData<Claims> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[issuer_url.trim_end_matches('/')]);
    validation.set_audience(&["account".to_string()]);
    validation.insecure_disable_signature_validation();

    let decoding_key = DecodingKey::from_secret(&[]);

    jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation).expect("failed to decode")
}
