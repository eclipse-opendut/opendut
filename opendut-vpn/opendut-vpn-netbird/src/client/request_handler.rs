use async_trait::async_trait;
use reqwest::{Request, Response};

use crate::netbird::error::RequestError;

#[async_trait]
pub(super) trait RequestHandler {
    async fn handle(&self, request: Request) -> Result<Response, RequestError>;
}

pub(super) struct DefaultRequestHandler {
    inner: reqwest::Client,
}

#[async_trait]
impl RequestHandler for DefaultRequestHandler {
    async fn handle(&self, request: Request) -> Result<Response, RequestError> {
        self.inner.execute(request).await.map_err(RequestError::Request)
    }
}

impl From<reqwest::Client> for DefaultRequestHandler {
    fn from(value: reqwest::Client) -> Self {
        Self { inner: value }
    }
}
