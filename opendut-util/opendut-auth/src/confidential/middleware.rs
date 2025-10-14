use http::Extensions;
use crate::confidential::client::ConfidentialClientRef;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};

#[derive(Clone)]
pub struct OAuthMiddleware {
    confidential_client: ConfidentialClientRef,
}

impl OAuthMiddleware {
    pub fn new(confidential_client: ConfidentialClientRef) -> Self {
        Self { confidential_client }
    }
}

#[async_trait::async_trait]
impl Middleware for OAuthMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>
    ) -> reqwest_middleware::Result<Response> {
        if let Ok(token) = self.confidential_client.get_token().await {
            req.headers_mut().insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", token.value).parse().unwrap(),
            );
        }
        let resp = next.run(req, extensions).await?;

        Ok(resp)
    }
}


