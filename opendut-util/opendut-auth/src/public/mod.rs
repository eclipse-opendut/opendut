pub use leptos_oidc::Auth;
use tonic::service::Interceptor;
use tonic::Status;

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
        match &self.auth {
            Some(auth) => {
                match auth.access_token() {
                    None => {
                        tracing::debug!("AuthInterceptor: No access token present.");
                        Err(Status::unauthenticated("No access token present."))
                    }
                    Some(token) => {
                        let bearer_token: tonic::metadata::MetadataValue<_> = format!("Bearer {}", token).parse()
                            .map_err(|_err| Status::unauthenticated("could not parse token"))?;
                        request.metadata_mut().insert(http::header::AUTHORIZATION.as_str(), bearer_token);
                        Ok(request)
                    }
                }
            }
            None => Ok(request)
        }
    }
}

#[derive(Clone, Debug)]
pub struct OptionalAuthData {
    pub auth_data: Option<AuthData>,
}

#[derive(Clone, Debug)]
pub struct AuthData {
    pub preferred_username: String,
    pub name: String,
    pub email: String,
    pub groups: Vec<String>,
    pub roles: Vec<String>,
    pub subject: String, // User identity in keycloak
}
