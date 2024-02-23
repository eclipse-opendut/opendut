use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Request, Response};
use reqwest_middleware::{ClientBuilder, Middleware, Next};
use reqwest_retry::{RetryTransientMiddleware};
use reqwest_retry::policies::ExponentialBackoff;
use task_local_extensions::Extensions;
use tracing::trace;

use crate::netbird::error::RequestError;

#[async_trait]
pub trait RequestHandler {
    async fn handle(&self, request: Request) -> Result<Response, RequestError>;
}

pub struct DefaultRequestHandler {
    inner: reqwest::Client,
    config: RequestHandlerConfig,
}

impl DefaultRequestHandler {
    pub fn new(inner: reqwest::Client, config: RequestHandlerConfig) -> Self {
        Self { inner, config }
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
        let resp = next.run (req, extensions).await?;
        trace!("Got response {}", resp.status());
        Ok(resp)
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

    pub fn load(config: &config::Config) -> Result<Self, opendut_util::settings::LoadError> {
        let timeout = Duration::from_millis(
            config.get::<u64>("vpn.netbird.timeout.ms")?
        );
        let retries = config.get::<u32>("vpn.netbird.retries")?;

        Ok(RequestHandlerConfig {
            timeout,
            retries
        })
    }
}