use crate::auth::grpc_auth_layer::GrpcAuthenticationLayer::GrpcAuthLayerEnabled;
use crate::auth::json_web_key::JwkCacheValue;
use crate::auth::validation::{authorize_user, Jwk, ValidationError};
use crate::auth::CurrentUser;
use crate::util::in_memory_cache::CustomInMemoryCache;
use tonic::Status;
use tracing::debug;
use url::Url;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum GrpcAuthenticationLayer {
    AuthDisabled,
    GrpcAuthLayerEnabled {
        issuer_url: Url,
        issuer_remote_url: Url,
        cache: CustomInMemoryCache<String, JwkCacheValue>,
    },
}

impl GrpcAuthenticationLayer {
    pub async fn auth_interceptor(self, mut request: tonic::Request<()>) -> anyhow::Result<tonic::Request<()>, Status> {

        match self {
            GrpcAuthenticationLayer::AuthDisabled => {
                Ok(request)
            }
            GrpcAuthLayerEnabled { issuer_url, issuer_remote_url, cache } => {
                let auth_header = match request.metadata().get("authorization") {
                    None => {
                        return Err(Status::unauthenticated("CARL says, you did not provide credentials!"))
                    }
                    Some(token) => {
                        token.to_str()
                            .map_err(|_| Status::unauthenticated("CARL says, your credentials are malformed!"))?
                    }
                };

                match authorize_current_user(auth_header, issuer_url, issuer_remote_url, cache).await {
                    Ok(user) => {
                        debug!("User: {:?} - Claims: {:?}", user.name, user.claims);

                        request.extensions_mut().insert(user);
                        Ok(request)
                    }
                    Err(cause) => {
                        debug!("Blocking authentication attempt due to error while validating credentials: {cause}");
                        Err(Status::unauthenticated("CARL says, invalid credentials!"))
                    }
                }
            }
        }
    }
}

async fn authorize_current_user(auth_token: &str, issuer_url: Url, issuer_remote_url: Url, cache: CustomInMemoryCache<String, JwkCacheValue>) -> Result<CurrentUser, ValidationError> {
    // decode token
    let token_part: Vec<&str> = auth_token.split(' ').collect();
    let token_part = token_part.get(1).unwrap();

    let jwk_requester = Jwk;
    authorize_user(issuer_url, issuer_remote_url, token_part, cache, jwk_requester, false).await
}
