#[cfg(feature = "client")]
mod native;
#[cfg(feature = "client")]
pub use native::*;

#[cfg(feature = "wasm-client")]
pub mod wasm;

use std::fmt::Display;
use tonic::codegen::http::uri::InvalidUri;
use opendut_util::proto::ConversionError;


#[derive(thiserror::Error, Debug)]
pub enum ClientError<A>
where
    A: Display
{
    #[error("{0}")]
    TransportError(String),
    #[error("{0}")]
    InvalidRequest(String),
    #[error("{0}")]
    InvalidResponse(String),
    #[error("{0}")]
    UsageError(A),
}

impl <A> From<tonic::Status> for ClientError<A>
where
    A: Display
{
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            tonic::Code::InvalidArgument => {
                Self::InvalidRequest(status.message().to_owned())
            }
            _ => {
                Self::TransportError(status.message().to_owned())
            }
        }
    }
}

impl <A> From<ConversionError> for ClientError<A>
where
    A: Display
{
    fn from(cause: ConversionError) -> Self {
        Self::InvalidResponse(cause.to_string())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum InitializationError {
    #[error("Invalid URI '{uri}': {cause}")]
    InvalidUri { uri: String, cause: InvalidUri },
    #[error("Expected https scheme. Given scheme: '{given_scheme}'")]
    ExpectedHttpsScheme { given_scheme: String },
    #[error("{message}: {cause}")]
    OidcConfiguration { message: String, cause: Box<dyn std::error::Error + Send + Sync> },
    #[error("{message}: {cause}")]
    TlsConfiguration { message: String, cause: Box<dyn std::error::Error + Send + Sync> },
    #[error("Error while connecting to CARL at '{address}': {cause}")]
    ConnectError { address: String, cause: Box<dyn std::error::Error + Send + Sync> },
}

pub trait ExtractOrClientError<A, B, E>
where
    B: TryFrom<A>,
    B::Error: Display,
    E: Display
{
    fn extract_or_client_error(self, field: impl Into<String> + Clone) -> Result<B, ClientError<E>>;
}

impl <A, B, E> ExtractOrClientError<A, B, E> for Option<A>
where
    B: TryFrom<A>,
    B::Error: Display,
    E: Display
{
    fn extract_or_client_error(self, field: impl Into<String> + Clone) -> Result<B, ClientError<E>> {
        self
            .ok_or_else(|| ClientError::InvalidResponse(format!("Field '{}' not set", Clone::clone(&field).into())))
            .and_then(|value| {
                B::try_from(value)
                    .map_err(|cause| ClientError::InvalidResponse(format!("Field '{}' is not valid: {}", field.into(), cause)))
            })
    }
}


macro_rules! extract {
    ($spec:expr) => {
        crate::carl::ExtractOrClientError::extract_or_client_error($spec, stringify!($spec))
    };
}

pub(crate) use extract;


use std::pin::Pin;
use tonic::codegen::tokio_stream::Stream;

pub struct GrpcDownstream<T> {
    inner: Box<dyn Stream<Item=Result<T, tonic::Status>> + Send + Unpin>,
}
impl<T> GrpcDownstream<T> {
    pub async fn receive(&mut self) -> Result<Option<T>, tonic::Status> {
        match std::future::poll_fn(|cx| Pin::new(&mut *self.inner).poll_next(cx)).await {
            Some(Ok(m)) => Ok(Some(m)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}
impl<T, S: Stream<Item=Result<T, tonic::Status>> + Send + Unpin + 'static> From<S> for GrpcDownstream<T> {
    fn from(value: S) -> Self {
        Self { inner: Box::new(value) }
    }
}
