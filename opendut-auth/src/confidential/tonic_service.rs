use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use http::{HeaderValue, Request, Response};
use tonic::body::Body;
use tonic::transport::Channel;
use tower::Service;
use tracing::error;
use crate::confidential::client::ConfidentialClient;

#[derive(Clone)]
pub struct TonicAuthenticationService {
    inner: Channel,
    confidential_client: Option<Arc<ConfidentialClient>>,
}

impl TonicAuthenticationService {
    pub fn new(
        inner: Channel,
        confidential_client: Option<Arc<ConfidentialClient>>,
    ) -> Self {
        TonicAuthenticationService {
            inner,
            confidential_client,
        }
    }
}

impl Service<Request<Body>> for TonicAuthenticationService {
    type Response = Response<Body>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, mut request: Request<Body>) -> Self::Future {
        // Combining a tower Service with a tonic client panics #547
        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let confidential_client = self.confidential_client.clone();

        Box::pin(async move {
            let token_result = confidential_client.as_ref()
                .map(|manager| manager.get_token());

            match token_result {
                None => {
                    // Authentication disabled
                    Ok(inner.call(request).await?)
                }
                Some(token_future) => {
                    // Authentication enabled
                    let token = token_future.await;
                    match token {
                        Ok(token) => {
                            let token = token.to_string();

                            let bearer_header =
                                HeaderValue::from_str(format!("Bearer {}", token.as_str()).as_str())
                                    .unwrap();

                            request.headers_mut().insert("Authorization", bearer_header);
                            let response = inner.call(request).await?;
                            Ok(response)
                        }
                        Err(error) => {
                            error!("Failed to get token: {}", error);
                            Err(error.into())
                        }
                    }
                }
            }
        })
    }
}
