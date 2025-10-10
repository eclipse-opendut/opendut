use std::time::Duration;

use crate::netbird::error::RequestError;
use async_trait::async_trait;
use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{ClientBuilder, Middleware, Next};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use tracing::trace;
use crate::NetbirdToken;

#[async_trait]
pub trait RequestHandler {
    async fn handle(&self, request: Request) -> Result<Response, RequestError>;
}

pub struct DefaultRequestHandler {
    inner: reqwest::Client,
    config: RequestHandlerConfig,
    netbird_token: NetbirdToken,
}

impl DefaultRequestHandler {
    pub fn new(inner: reqwest::Client, config: RequestHandlerConfig, netbird_token: NetbirdToken) -> Self {
        Self { inner, config, netbird_token }
    }
}

#[async_trait]
impl RequestHandler for DefaultRequestHandler {
    async fn handle(&self, mut request: Request) -> Result<Response, RequestError> {

        let timeout = request.timeout_mut()
            .get_or_insert(self.config.timeout);

        trace!("Starting network request with timeout {} milliseconds.", timeout.as_millis());
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(self.config.retries);
        let client = ClientBuilder::new(self.inner.to_owned())
            .with(AuthorizationHeaderMiddleware::new(self.netbird_token.clone()))
            .with(LoggingMiddleWare)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        let result = client.execute(request).await.map_err(RequestError::RequestMiddleware);
        trace!("Network request completed.");

        result
    }
}
pub struct RequestHandlerConfig {
    pub timeout: Duration,
    pub retries: u32,
}

impl RequestHandlerConfig {
    pub fn new(timeout: Duration, retries: u32) -> Self {
        Self {
            timeout,
            retries
        }
    }
}

struct LoggingMiddleWare ;

#[async_trait::async_trait]
impl Middleware for LoggingMiddleWare {
    async fn handle (
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        trace!("Sending request {} {}", req.method(), req.url());
        let resp = next.run(req, extensions).await?;
        trace!("Got response {}", resp.status());
        Ok(resp)
    }
}

#[derive(Clone)]
pub struct AuthorizationHeaderMiddleware {
    api_token: NetbirdToken,
}

impl AuthorizationHeaderMiddleware {
    pub fn new(api_token: NetbirdToken) -> Self {
        Self { api_token }
    }
}

#[async_trait::async_trait]
impl Middleware for AuthorizationHeaderMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>
    ) -> reqwest_middleware::Result<Response> {
        println!("Inserting token into request: {:?}", self.api_token);
        req.headers_mut().insert(
            reqwest::header::AUTHORIZATION,
            self.api_token.sensitive_header()
                .map_err(|error| reqwest_middleware::Error::Middleware(anyhow::anyhow!(error)))?,
        );
        let resp = next.run(req, extensions).await?;

        Ok(resp)
    }
}


