use tonic::Status;
use url::Url;
use crate::auth::grpc_auth_layer::GrpcAuthenticationLayer::GrpcAuthLayerEnabled;
use crate::auth::json_web_key::JwkCacheValue;
use crate::util::in_memory_cache::CustomInMemoryCache;

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
                        token.to_str().map_err(|_| Status::unauthenticated("CARL says, your credentials are malformed!"))?
                    }
                };

                if let Some(current_user) = crate::auth::authorization::authorize_current_user(auth_header, issuer_url, issuer_remote_url, cache).await {
                    // insert the current user info into a request extension
                    request.extensions_mut().insert(current_user);
                    Ok(request)
                } else {
                    Err(Status::unauthenticated("CARL says, invalid credentials!"))
                }
            }
        }
    }
}
