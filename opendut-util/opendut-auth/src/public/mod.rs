use std::ops::Sub;
pub use leptos_oidc::Auth;
use tonic::service::Interceptor;
use tonic::Status;
use crate::TOKEN_GRACE_PERIOD;

#[derive(Clone)]
pub struct AuthInterceptor {
    auth: Option<Auth>,
}

impl AuthInterceptor {
    pub fn new(auth: Option<Auth>) -> Self {
        Self { auth }
    }
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        if let Some(auth) = &self.auth {
            if let Some(Some(token_storage)) = auth.ok() {
                let now = chrono::Utc::now().naive_utc();
                if now.gt(&token_storage.expires_in.sub(TOKEN_GRACE_PERIOD)) {
                    tracing::debug!("Token expired. Refreshing.");
                    auth.refresh_token();
                }
            };

            let token = match auth.access_token() {
                None => { "no-auth-token".to_string() }
                Some(token) => { token }
            };
            tracing::debug!("Token: {}", token);
            let token: tonic::metadata::MetadataValue<_> = format!("Bearer {}", token).parse()
                .map_err(|_err| Status::unauthenticated("could not parse token"))?;
            request.metadata_mut().insert("authorization", token.clone());
        }

        Ok(request)
    }
}

#[derive(Clone, Debug)]
pub struct OptionalAuthData {
    pub auth_data: Option<AuthData>,
}

#[derive(Clone, Debug)]
pub struct AuthData {
    pub access_token: String,
    pub preferred_username: String,
    pub name: String,
    pub email: String,
    pub groups: Vec<String>,
    pub roles: Vec<String>,
    pub subject: String,
}
