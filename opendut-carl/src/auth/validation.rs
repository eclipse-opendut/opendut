use std::collections::BTreeMap;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use crate::auth::{Claims, CurrentUser, MyAdditionalClaims};
use crate::auth::json_web_key::{JsonWebKey, JwkCacheValue, OidcJsonWebKeySet};
use url::Url;
use crate::util::in_memory_cache::CustomInMemoryCache;

pub const GIVEN_ISSUER_JWK_URL: &str = "protocol/openid-connect/certs";

#[derive(thiserror::Error, Debug, Clone, Deserialize, Serialize)]
pub enum ValidationError {
    #[error("Configuration Error: {0}")]
    Configuration(String),
    #[error("Algo Error: {0}")]
    InvalidAlgorithm(String),
    #[error("Fatal validation Error: {0}")]
    Failed(String),
    #[error("Failed to process cache: {0}")]
    Cache(String),
}

pub fn validate_token(issuer_remote_url: Url, access_token: &str, jwk: JsonWebKey, validate_expiration: bool) -> Result<CurrentUser, ValidationError> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[issuer_remote_url.as_str().trim_end_matches('/')]);
    // TODO: evaluate if account audience is appropriate
    validation.set_audience(&["account".to_string()]);
    validation.validate_exp = validate_expiration;

    let decoding_key = DecodingKey::from_rsa_components(&jwk.modulus, &jwk.exponent).expect("failed to create decoding key");

    let token = jsonwebtoken::decode::<Claims<MyAdditionalClaims>>(access_token, &decoding_key, &validation)
        .map_err(|err| ValidationError::Failed(format!("failed to decode token: {}", err)))?;

    let username = match token.claims.preferred_username() {
        None => { return Err(ValidationError::Configuration("Missing preferred username".to_string())); }
        Some(username) => { username.to_string() }
    };

    Ok(CurrentUser {
        name: username,
        claims: token.claims,
    })
}

pub async fn authorize_user(issuer_url: Url, issuer_remote_url: Url, access_token: &str, cache: CustomInMemoryCache<String, JwkCacheValue>, jwk_requester: impl JwkRequester, token_validate_expiration: bool) -> Result<CurrentUser, ValidationError> {
    
    // TODO:  additional DDOS prevention? token contains unknown kid -> refetch jwk, kid still unknown, malicious DDOS?, blacklist kid?

    let key_id = get_key_id(access_token)?;

    let jwk = match cache.get(&key_id) {
        Ok(Some(jwk_cache_value)) => {
            let timestamp_now = Utc::now().timestamp();
            if timestamp_now - jwk_cache_value.last_cached > Duration::days(1).num_seconds() {
                fetch_and_cache_jwk_from_idp(&issuer_url, key_id, jwk_requester, cache).await?
            } else {
                jwk_cache_value.jwk
            }
        }
        Ok(None) => {
            fetch_and_cache_jwk_from_idp(&issuer_url, key_id, jwk_requester, cache).await?
        }
        Err(error) => {
            return Err(ValidationError::Cache(format!("Failed to get cache entry: {}", error)));
        }
    };

    let validate_with_remote_issuer = validate_token(issuer_remote_url, access_token, jwk.clone(), token_validate_expiration);
    match validate_with_remote_issuer {
        Ok(user) => {
            Ok(user)
        }
        Err(error) => {
            // Fallback for dev/test environment
            if error.to_string().contains("InvalidIssuer") {
                validate_token(issuer_url, access_token, jwk, token_validate_expiration)
            } else {
                Err(error)
            }
        }
    }
}

async fn fetch_and_cache_jwk_from_idp(issuer_url: &Url, key_id: String, jwk_requester: impl JwkRequester, mut cache: CustomInMemoryCache<String, JwkCacheValue>) -> Result<JsonWebKey, ValidationError> {
    let issuer_jwk_url = issuer_url.join(GIVEN_ISSUER_JWK_URL)
        .map_err(|cause| ValidationError::Configuration(format!("Issuer JWK error: {cause}")))?;

    let jwk_map = fetch_jwk_custom(issuer_jwk_url, jwk_requester).await?;

    if let Err(error) = cache.delete_all() {
        return Err(ValidationError::Cache(format!("Failed to evict cache: {}", error)));
    }

    for (kid, jwk) in &jwk_map {
        let jwk_cache_value = JwkCacheValue {
            jwk: Clone::clone(jwk),
            last_cached: Utc::now().timestamp()
        };
        if let Err(error) = cache.insert(String::from(kid), jwk_cache_value) {
            return Err(ValidationError::Cache(format!("Failed to store jwk: {}", error)));
        }
    };

    let json_web_key = match jwk_map.get(&key_id) {
        None => { return Err(ValidationError::InvalidAlgorithm(format!("Could not find key id: {}", key_id))); }
        Some(jwk) => { jwk.clone() }
    };
    Ok(json_web_key)
}

fn get_key_id(access_token: &str) -> Result<String, ValidationError> {
    let header = jsonwebtoken::decode_header(access_token)
        .map_err(|error| ValidationError::Configuration(format!("Failed to decode header: {}", error)))?;

    // TODO: valid algorithms are determined by the issuer (fetch keycloak certificates / jwk)
    match header.alg {
        Algorithm::RS256 => {}
        _ => { return Err(ValidationError::InvalidAlgorithm(format!("Could not handle algorithm: {:?}", header.alg))); }
    }

    match header.kid {
        None => { Err(ValidationError::InvalidAlgorithm("Missing key id".to_string())) }
        Some(kid) => { Ok(kid) }
    }
}

async fn fetch_jwk_custom(issuer_jwk_url: Url, jwk_requester: impl JwkRequester) -> Result<BTreeMap<String, JsonWebKey>, ValidationError> {
    let result = jwk_requester.fetch_jwk(issuer_jwk_url).await?;
    let json_web_key_set = serde_json::from_str::<OidcJsonWebKeySet>(result.as_str())
        .map_err(|cause| ValidationError::Configuration(format!("Failed to parse json: {}", cause)))?;
    let jwk_map = json_web_key_set.keys.into_iter().map(|jwk| {
        (jwk.key_identifier.clone(), jwk)
    }).collect::<BTreeMap<_, _>>();
    Ok(jwk_map)
}

pub trait JwkRequester {
    async fn fetch_jwk(&self, issuer_jwk_url: Url) -> Result<String, ValidationError>;
}

pub struct Jwk;

impl JwkRequester for Jwk {
    async fn fetch_jwk(&self, issuer_jwk_url: Url) -> Result<String, ValidationError> {
        let response = reqwest::get(issuer_jwk_url.clone()).await.map_err(|cause| ValidationError::Configuration(format!("Failed to fetch IDP jwk URL from '{}': {}", issuer_jwk_url, cause)))?;
        let result = response.text().await.map_err(|cause| ValidationError::Configuration(format!("Failed to read IDP configuration URL from '{}': {}", issuer_jwk_url, cause)))?;
        Ok(result)
    }
}


#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::ops::Sub;
    use chrono::{Duration, Utc};
    use googletest::{assert_that};
    use googletest::matchers::{contains_substring};
    use rstest::{fixture, rstest};
    use url::Url;
    use crate::auth::json_web_key::{JsonWebKey, JwkCacheValue, OidcJsonWebKeySet};
    use crate::util::in_memory_cache::CustomInMemoryCache;
    use crate::auth::validation::{authorize_user, JwkRequester, validate_token, ValidationError};

    const KEY_ID: &str = "9RcB1okOXQ6QibEeXzAxFVym9PmBynkFe8mbh6X-DB0";
    const TEST_TOKEN: &str = "eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICI5UmNCMW9rT1hRNlFpYkVlWHpBeEZWeW05UG1CeW5rRmU4bWJoNlgtREIwIn0.eyJleHAiOjE3MjE3MjUzMjgsImlhdCI6MTcyMTcyNTAyOCwiYXV0aF90aW1lIjoxNzIxNzI1MDI4LCJqdGkiOiJkNzI1ZjlkZi04OWQ5LTQwZGQtOGMzYi0yMzA2ZGUzNzkzODUiLCJpc3MiOiJodHRwczovL2tleWNsb2FrL3JlYWxtcy9vcGVuZHV0IiwiYXVkIjoiYWNjb3VudCIsInN1YiI6IjljNTBhOGU5LTRlZjYtNGE4Zi04ZDZlLWFkNjhiYjk4NGJhMSIsInR5cCI6IkJlYXJlciIsImF6cCI6Im9wZW5kdXQtbGVhLWNsaWVudCIsInNlc3Npb25fc3RhdGUiOiIzZGYxZGM5YS1jMjMzLTRiMWEtODdlYS1kMGYyOTVlMDBmNzUiLCJhY3IiOiIxIiwiYWxsb3dlZC1vcmlnaW5zIjpbIioiXSwicmVzb3VyY2VfYWNjZXNzIjp7ImFjY291bnQiOnsicm9sZXMiOlsibWFuYWdlLWFjY291bnQiLCJtYW5hZ2UtYWNjb3VudC1saW5rcyIsInZpZXctcHJvZmlsZSJdfX0sInNjb3BlIjoib3BlbmlkIGVtYWlsIHByb2ZpbGUgZ3JvdXBzIiwic2lkIjoiM2RmMWRjOWEtYzIzMy00YjFhLTg3ZWEtZDBmMjk1ZTAwZjc1IiwiZW1haWxfdmVyaWZpZWQiOmZhbHNlLCJyb2xlcyI6WyJvZmZsaW5lX2FjY2VzcyIsImRlZmF1bHQtcm9sZXMtb3BlbmR1dCIsInRlc3Ryb2xlIiwidW1hX2F1dGhvcml6YXRpb24iXSwibmFtZSI6IkZpcnN0bmFtZSBMYXN0bmFtZSIsImdyb3VwcyI6WyIvdGVzdGdyb3VwIl0sInByZWZlcnJlZF91c2VybmFtZSI6Im9wZW5kdXQiLCJnaXZlbl9uYW1lIjoiRmlyc3RuYW1lIiwiZmFtaWx5X25hbWUiOiJMYXN0bmFtZSIsImVtYWlsIjoib3BlbmR1dEBleGFtcGxlLmNvbSJ9.PLYTZ_v4GGM6YPZC_afI67eJ8U5sbV6aS2YbBDhmvNfhH-g-Sn_2NZImcPLxiz50_5pbRhhi8pnDnshbLHkxv2uEj1ltdPRmSCD4xqzlP7kDLn0kMVsBJHIeL5olj7zY8KjWJAieFH2oOZIiMiWRAsD9SAUSyr1tTNv38p6i0Pyy_Op-fDlF1zZel2adLke8j0Svb7H63OSsOTt8HES-sUIMd4VJDH3yb83OECFVBEieE3GRq_77BgtffzgXgJZAAA84ija7O-ao_raSoy1ycqykEqmdSu9X-dzw_YrjtroUBM7RS4hrI9iJ5pGwH_LESUd8L93xUX5yYEZeN-0r-g";
    const ISSUER_URL: &str  = "https://keycloak/realms/opendut/";
    const JWK_RAW_DATA: &str = r#"{"keys":[{"kid":"9RcB1okOXQ6QibEeXzAxFVym9PmBynkFe8mbh6X-DB0","kty":"RSA","alg":"RS256","use":"sig","n":"jJTeGo90wWqXEk4JHRlPVF5hOXViKk5qnIlwiUAyx3CfBBuwSVEKVCq73TtuG57EQFca-o01SYKGGg-yU2VyleEDKbSGBzdl2LelrUwHCdSphupnIGPJ12wU8EDBgfOh0llWpNYTrEtNjbHLaYbMZL9_a7sXOTJxC6-S9EcpyhvI0LZHjOJe_YAnkj1Wx5OKWRZhiV5_y00SQI8xHinnOKLWH86giOBBJuN5Z-Ii3xNPF8jtHLdEXNw6cbeueaeU56Rlmy9AkuGdnQzBnP4hMRVul7Poam7iDD30Rl_qfH4yO-jhDnw1Mz4JALBPToaZ3WC6oXkfoGQo0Q4wmN3oNQ","e":"AQAB","x5c":["MIICnTCCAYUCBgGQgfqpwDANBgkqhkiG9w0BAQsFADASMRAwDgYDVQQDDAdvcGVuZHV0MB4XDTI0MDcwNTA4MTgyNloXDTM0MDcwNTA4MjAwNlowEjEQMA4GA1UEAwwHb3BlbmR1dDCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAIyU3hqPdMFqlxJOCR0ZT1ReYTl1YipOapyJcIlAMsdwnwQbsElRClQqu907bhuexEBXGvqNNUmChhoPslNlcpXhAym0hgc3Zdi3pa1MBwnUqYbqZyBjyddsFPBAwYHzodJZVqTWE6xLTY2xy2mGzGS/f2u7FzkycQuvkvRHKcobyNC2R4ziXv2AJ5I9VseTilkWYYlef8tNEkCPMR4p5zii1h/OoIjgQSbjeWfiIt8TTxfI7Ry3RFzcOnG3rnmnlOekZZsvQJLhnZ0MwZz+ITEVbpez6Gpu4gw99EZf6nx+Mjvo4Q58NTM+CQCwT06Gmd1guqF5H6BkKNEOMJjd6DUCAwEAATANBgkqhkiG9w0BAQsFAAOCAQEAczQGabVZrMKsJNV+eoLcCUxzLv9tYRaFbLrT5+keotgl6YYfZ3W63wY9IaZp0wT5zKdG2meifJ48173VP/8/437A+t0zCkH2kfQY9sP3EXDKVbw8LuViaoVO2w3GoanRJP8BKSAMo3voRCnd6QAPCbaTIUM2M0bRl1RADRuAZXbWM8817Sk2w0qMkSyxDJY9JNRviUQBU0V4ziro9mB+pVIMJ/Z4anNGsTNL6D9HdI3/7iBuC7SLTVh8x/Yg0mYnud8WwRePOZuxDbA65V2lL3ixB4uhjq9yuo5F76c/TuyrFFUrXXmUMn5+0/OjRhHEKBZSUJHGvvQlgkjzkOcovg=="],"x5t":"pa3zfyZhNzSUhKHXzIn5QbOuFyA","x5t#S256":"v8an46MZ8wHfjnUW2fUGl5Xh602pXEC8Lb_p7EUSATg"},{"kid":"rSPOu3JnH_GrUFiekXboNx7s4xO816XM7Hb_F8bz8Y0","kty":"RSA","alg":"RSA-OAEP","use":"enc","n":"kKo_9nNiiLcImSd5xdNFEUEaQ6BFe9j__XOdEaFNMfa0zc-lu4J6wjyDEILR5HdgzQfaRlne66z4TwiJwyoyDRz7EqB75voagmsZn9UK8CGp4h27Tz7y1doPletRV3458PWPzy4epYAgsu-yEYVXTc8OT_XnXlnNAN4z1DpI-1Kk4uFS1zvRUiUvr8kzauJbPdA7LTKMU5vw5yfjATMZL3ZlhwNLnU82xqr4zqnMdrAeQewuGEXud8-IUHotTKCuM-KwkRjLRrIxYNMyM9h8UStOXpxlc8ARwyrjWGfFVbUPNlxossSzLP223OiCEBY_SEDF8d9gsl7NkSAJdOUE6w","e":"AQAB","x5c":["MIICnTCCAYUCBgGQgfqq2jANBgkqhkiG9w0BAQsFADASMRAwDgYDVQQDDAdvcGVuZHV0MB4XDTI0MDcwNTA4MTgyN1oXDTM0MDcwNTA4MjAwN1owEjEQMA4GA1UEAwwHb3BlbmR1dDCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAJCqP/ZzYoi3CJknecXTRRFBGkOgRXvY//1znRGhTTH2tM3PpbuCesI8gxCC0eR3YM0H2kZZ3uus+E8IicMqMg0c+xKge+b6GoJrGZ/VCvAhqeIdu08+8tXaD5XrUVd+OfD1j88uHqWAILLvshGFV03PDk/1515ZzQDeM9Q6SPtSpOLhUtc70VIlL6/JM2riWz3QOy0yjFOb8Ocn4wEzGS92ZYcDS51PNsaq+M6pzHawHkHsLhhF7nfPiFB6LUygrjPisJEYy0ayMWDTMjPYfFErTl6cZXPAEcMq41hnxVW1DzZcaLLEsyz9ttzoghAWP0hAxfHfYLJezZEgCXTlBOsCAwEAATANBgkqhkiG9w0BAQsFAAOCAQEAQsLO8nRuRGl5YqV0IJaX4GDunc7EGfD4Gofl5NNtG3SojISC0lmO4EyZdFsXJmmWgzFkg1aO91jdcZyIaf6qBbj+GPtoBltA0+nSAcCTDvOsmV1J1Gymxm/CJLTBGqIrLwEXDBFyFpF2W7OE7XdXby+d/mYVkpCc0fHC854w+tOLdvEr4AYD/3JNK5VWd1RLI1CeZ7nJeLbDUR5UkGGb2Na3SXaEsWWwor2L9OAY4bWq9+gIom7ihaDvXMMpMHbQ7gis8Ku5ltK80PISW/9b+G1IxKNYy+euCr9ZWiIeEcKBt0/dKSvCcfhG0mShmliETgGAfAdZu0eqqhuxATAi9A=="],"x5t":"gfshQCGXfVblp5YrHiYSlYUto90","x5t#S256":"h53Q-c8zYde1UjhjhLZB1I5Q7tjX-t9bz7lE_fV6Bbg"}]}"#;

    #[rstest]
    fn test_validate_token(fixture: Fixture) {
        let result = validate_token(fixture.issuer_url, TEST_TOKEN, fixture.jwk, false)
            .map_err(|err| println!("Failed to get current user: {:?}", err));
        assert!(result.is_ok());
    }

    #[rstest]
    fn test_validate_expired_token(fixture: Fixture) {
        let result = validate_token(fixture.issuer_url, TEST_TOKEN, fixture.jwk, true);
        assert!(result.is_err());
        assert_that!(result.err().unwrap().to_string(), contains_substring("ExpiredSignature"));
    }

    #[rstest]
    #[tokio::test]
    async fn test_authorize_user(fixture: Fixture) {
        let cache: CustomInMemoryCache<String, JwkCacheValue> = CustomInMemoryCache::new();
        let jwk_requester = MockJwk { jwk: String::from(JWK_RAW_DATA) };

        let result = authorize_user(
            fixture.issuer_url, 
            fixture.issuer_remote_url, 
            TEST_TOKEN, 
            cache.clone(), 
            jwk_requester, false
        ).await;
        
        assert!(result.is_ok());
        assert!(cache.get(&fixture.key_id).is_ok())
    }

    #[rstest]
    #[tokio::test]
    async fn test_authorize_user_jwk_already_cached(fixture: Fixture) {
        let jwk_requester = MockJwkError { };

        let result = authorize_user(
            fixture.issuer_url, 
            fixture.issuer_remote_url, 
            TEST_TOKEN,
            fixture.up_to_date_cache.clone(), 
            jwk_requester, 
            false
        ).await;
        
        assert!(result.is_ok());
        assert!(fixture.up_to_date_cache.get(&fixture.key_id).is_ok())
    }

    #[rstest]
    #[tokio::test]
    async fn test_authorize_user_with_jwk_cached_two_days_ago(fixture: Fixture) {
        let jwk_requester = MockJwk { jwk: String::from(JWK_RAW_DATA) };

        let result = authorize_user(
            fixture.issuer_url,
            fixture.issuer_remote_url,
            TEST_TOKEN,
            fixture.two_day_old_cache.clone(),
            jwk_requester,
            false
        ).await;
        
        assert!(result.is_ok());
        assert!(fixture.two_day_old_cache.get(&fixture.key_id).is_ok())
    }

    struct MockJwk {
        jwk: String
    }

    impl JwkRequester for MockJwk {
        async fn fetch_jwk(&self, _issuer_jwk_url: Url) -> Result<String, ValidationError> {
            Ok(self.jwk.clone())
        }
    }

    struct MockJwkError {}

    impl JwkRequester for MockJwkError{
        async fn fetch_jwk(&self, _issuer_jwk_url: Url) -> Result<String, ValidationError> {
            Err(ValidationError::Cache(String::from("Use cached value")))
        }
    }

    struct Fixture {
        key_id: String,
        issuer_url: Url,
        issuer_remote_url: Url,
        jwk: JsonWebKey,
        up_to_date_cache: CustomInMemoryCache<String, JwkCacheValue>,
        two_day_old_cache: CustomInMemoryCache<String, JwkCacheValue>,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let key_id = String::from(KEY_ID);
        let issuer_url: Url = Url::parse(ISSUER_URL).unwrap();
        let issuer_remote_url: Url = Url::parse(ISSUER_URL).unwrap();
        
        let json_web_key_set = serde_json::from_str::<OidcJsonWebKeySet>(JWK_RAW_DATA).unwrap();
        let jwk_map = json_web_key_set.keys.into_iter().map(|jwk| {
            (jwk.key_identifier.clone(), jwk)
        }).collect::<BTreeMap<_, _>>();

        let jwk = jwk_map.get(KEY_ID).expect("test could not get key_id from jwk map");

        let mut up_to_date_cache: CustomInMemoryCache<String, JwkCacheValue> = CustomInMemoryCache::new();
        let jwk_cache_value = JwkCacheValue {
            jwk: Clone::clone(&jwk),
            last_cached: Utc::now().timestamp()
        };
        up_to_date_cache.insert(key_id.clone(), jwk_cache_value).expect("Could not cache jwk");
        
        let mut two_day_old_cache: CustomInMemoryCache<String, JwkCacheValue> = CustomInMemoryCache::new();
        let utc_two_days_ago = Utc::now().sub(Duration::days(2));
        let jwk_two_day_old_cache_value = JwkCacheValue {
            jwk: Clone::clone(&jwk),
            last_cached: utc_two_days_ago.timestamp()
        };
        two_day_old_cache.insert(key_id.clone(), jwk_two_day_old_cache_value).expect("Could not cache jwk");
        
        Fixture {
            key_id,
            issuer_url,
            issuer_remote_url,
            jwk: jwk.clone(),
            up_to_date_cache,
            two_day_old_cache
        }
    }
}
