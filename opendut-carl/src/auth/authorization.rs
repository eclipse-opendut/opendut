use tracing::debug;
use url::Url;
use crate::auth::{CurrentUser};
use crate::auth::json_web_key::JwkCacheValue;
use crate::util::in_memory_cache::CustomInMemoryCache;
use crate::auth::validation::{authorize_user, Jwk};

pub(crate) async fn authorize_current_user(auth_token: &str, issuer_url: Url, issuer_remote_url: Url, cache: CustomInMemoryCache<String, JwkCacheValue>) -> Option<CurrentUser> {
    // decode token
    let token_part: Vec<&str> = auth_token.split(' ').collect();
    let jwk_requester = Jwk;
    let result = authorize_user(issuer_url, issuer_remote_url, token_part.get(1).unwrap(), cache, jwk_requester, false).await
        .map_err(|err| format!("Failed to validate token: {:?}", err));
    
    match result {
        Ok(user) => {
            debug!("User: {:?} - Claims: {:?}", user.name, user.claims);
            Some(user)
        }
        Err(_) => { None }
    }
}
