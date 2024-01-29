use std::time::Duration;
use async_trait::async_trait;
use reqwest::{Request, Response};

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

#[async_trait]
impl RequestHandler for DefaultRequestHandler {
    async fn handle(&self, mut request: Request) -> Result<Response, RequestError> {

        let timeout = request.timeout_mut()
            .get_or_insert(self.config.default_timeout);

        log::trace!("Starting network request with timeout {} milliseconds.", timeout.as_millis());
        let result = self.inner.execute(request).await.map_err(RequestError::Request);
        log::trace!("Network request completed.");

        result
    }
}

pub struct RequestHandlerConfig {
    default_timeout: Duration,
}

impl Default for RequestHandlerConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(10),
        }
    }
}
