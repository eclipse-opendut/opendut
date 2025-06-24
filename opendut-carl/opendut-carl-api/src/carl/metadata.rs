#![cfg(any(feature = "client", feature = "wasm-client"))]

pub use client::*;

mod client {
    use tonic::codegen::{Body, Bytes, http, InterceptedService, StdError};

    use opendut_types::proto::util::VersionInfo;

    use crate::proto::services::metadata_provider;
    use crate::proto::services::metadata_provider::metadata_provider_client::MetadataProviderClient;

    #[derive(Clone, Debug)]
    pub struct MetadataProvider<T> {
        inner: MetadataProviderClient<T>,
    }

    impl<T> MetadataProvider<T>
    where T: tonic::client::GrpcService<tonic::body::Body>,
          T::Error: Into<StdError>,
          T::ResponseBody: Body<Data=Bytes> + Send + 'static,
          <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: MetadataProviderClient<T>) -> MetadataProvider<T> {
            MetadataProvider { inner }
        }

        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> MetadataProvider<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<tonic::body::Body>>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                >>::Error: Into<StdError> + Send + Sync,
        {
            let inner_client = MetadataProviderClient::new(InterceptedService::new(inner, interceptor));
            MetadataProvider {
                inner: inner_client
            }
        }

        pub async fn version(&mut self) -> Result<VersionInfo, VersionRequestError> {
            let request = tonic::Request::new(metadata_provider::VersionRequest {});

            match self.inner.version(request).await {
                Ok(response) => {
                    let version = response.into_inner()
                        .version_info
                        .ok_or(VersionRequestError { message: String::from("Response contains no version info!") })?;
                    Ok(version)
                },
                Err(status) => {
                    Err(VersionRequestError { message: format!("gRPC failure: {status}") })
                },
            }
        }
    }

    #[derive(thiserror::Error, Debug)]
    #[error("{message}")]
    pub struct VersionRequestError {
        message: String,
    }
}

pub mod version_compatibility {
    use super::*;
    use crate::carl::CarlClient;
    use std::ops::Not;
    use tracing::{info, warn};

    pub struct VersionCompatibilityInfo {
        pub own_version: &'static str,
        pub own_name: String,
        pub upgrade_hint: Option<String>,
    }

    pub async fn log_version_compatibility_with_carl(
        info: VersionCompatibilityInfo,
        carl: &mut CarlClient,
    ) -> Result<(), VersionCompatibilityError> {
        let VersionCompatibilityInfo { own_version, own_name, upgrade_hint } = info;

        let carl_version = carl.metadata.version().await?;

        let version_requirement = semver::VersionReq::parse(own_version)?; //Adds implicit caret (^) to requirement, due to how `semver` crate works

        let carl_version = semver::Version::parse(&carl_version.name)?;
        let own_version = semver::Version::parse(own_version)?;
        let upgrade_hint = match upgrade_hint {
            Some(hint) => format!("\n{hint}"),
            None => String::new(),
        };

        if version_requirement.matches(&carl_version).not() { //matches as described here: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#version-requirement-syntax
            warn!("The version of CARL ({carl_version}) is incompatible with {own_name} ({own_version}). This can lead to unintended behavior and crashes.{upgrade_hint}");
        }
        else if carl_version > own_version {
            info!("CARL has a compatible but newer version ({carl_version}). You may want to update {own_name} ({own_version}) to benefit from the newest bug fixes.{upgrade_hint}");
        } else {
            info!("CARL has version {carl_version}.");
        }

        Ok(())
    }

    #[derive(Debug, thiserror::Error)]
    #[error("Error while checking version compatibility with CARL")]
    pub enum VersionCompatibilityError {
        Request(#[from] VersionRequestError),
        SemVer(#[from] semver::Error),
    }
}
