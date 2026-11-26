use opendut_carl_api::carl::ClientError;
use opendut_carl_api::carl::viper::{CompilationErrorKind, GetViperSourceDescriptorError, GetViperTestRunDescriptorError, GetViperTestSuiteParametersError};
use opendut_model::viper::{ViperSourceId, ViperTestId, ViperTestSuiteParameters};
use leptos::prelude::{Get, LocalResource, Signal, Track};
use opendut_lea_components::ErrorText;
use opendut_model::viper::{ViperSourceDescriptor, ViperTestRunDescriptor};
use tracing::error;

use crate::app::use_app_globals;
use crate::util::client_error_description;

pub fn list_viper_source_descriptors(
    refetch_signal: Option<Signal<()>>,
) -> LocalResource<Result<Vec<ViperSourceDescriptor>, ErrorText>> {
    let globals = use_app_globals();
    let carl = globals.client;

    LocalResource::new(move || {
        refetch_signal.track();

        let mut carl = carl.clone();
        async move {
            let mut descriptors = carl.viper.list_viper_source_descriptors().await
                .map_err(|error| {
                    error!("Failed to request the list of VIPER sources: {error}");

                    ErrorText {
                        headline: String::from("Unable to load VIPER sources"),
                        description: client_error_description(
                            &error,
                            "The VIPER sources could not be loaded. Please check your connection and try again.",
                        ),
                    }
                })?;

            descriptors.sort_by(|a, b| {
                a.name.to_string().to_lowercase()
                    .cmp(&b.name.to_string().to_lowercase())
                    .then(a.id.to_string().cmp(&b.id.to_string()))
            });

            Ok(descriptors)
        }
    })
}

pub fn list_viper_test_descriptors(
    refetch_signal: Option<Signal<()>>,
) -> LocalResource<Result<Vec<ViperTestRunDescriptor>, ErrorText>> {
    let globals = use_app_globals();
    let carl = globals.client;

    LocalResource::new(move || {
        refetch_signal.track();

        let mut carl = carl.clone();
        async move {
            let mut descriptors = carl.viper.list_viper_test_run_descriptors().await
                .map_err(|error| {
                    error!("Failed to request the list of VIPER test descriptors: {error}");

                    ErrorText {
                        headline: String::from("Unable to load VIPER test descriptors"),
                        description: client_error_description(
                            &error,
                            "The VIPER test descriptors could not be loaded. Please check your connection and try again.",
                        ),
                    }
                })?;

            descriptors.sort_by(|a, b| {
                a.name.value().to_owned().to_lowercase()
                    .cmp(&b.name.value().to_owned().to_lowercase())
                    .then(a.id.to_string().cmp(&b.id.to_string()))
            });

            Ok(descriptors)
        }
    })
}

pub fn get_viper_source_descriptor(
    id: impl Into<Signal<ViperSourceId>>,
    refetch_signal: Option<Signal<()>>,
) -> LocalResource<Result<Option<ViperSourceDescriptor>, ErrorText>> {
    let id = id.into();
    let carl = use_app_globals().client;

    LocalResource::new(move || {
        let id = id.get();
        refetch_signal.track();

        let mut carl = carl.clone();
        async move {
            match carl.viper.get_viper_source_descriptor(id).await {
                Ok(descriptor) => Ok(Some(descriptor)),
                Err(ClientError::UsageError(GetViperSourceDescriptorError::SourceNotFound { .. })) => Ok(None),
                Err(error) => {
                    error!("Unable to load VIPER source descriptor: {error}");
                    Err(ErrorText {
                        headline: String::from("Unable to load VIPER source descriptor"),
                        description: client_error_description(
                            &error,
                            "The VIPER source descriptor could not be loaded. Please check your connection and try again.",
                        ),
                    })
                }
            }
        }
    })
}

pub fn get_viper_test_run_descriptor(
    id: impl Into<Signal<ViperTestId>>,
    refetch_signal: Option<Signal<()>>,
) -> LocalResource<Result<Option<ViperTestRunDescriptor>, ErrorText>> {
    let id = id.into();
    let carl = use_app_globals().client;

    LocalResource::new(move || {
        let id = id.get();
        refetch_signal.track();

        let mut carl = carl.clone();
        async move {
            match carl.viper.get_viper_test_run_descriptor(id).await {
                Ok(descriptor) => Ok(Some(descriptor)),
                Err(ClientError::UsageError(GetViperTestRunDescriptorError::TestNotFound { .. })) => Ok(None),
                Err(error) => {
                    error!("Unable to load VIPER test run descriptor: {error}");
                    Err(ErrorText {
                        headline: String::from("Unable to load VIPER test run descriptor"),
                        description: client_error_description(
                            &error,
                            "The VIPER test run descriptor could not be loaded. Please check your connection and try again.",
                        ),
                    })
                }
            }
        }
    })
}

pub fn get_viper_test_suite_parameters(
    id: impl Into<Signal<Option<ViperSourceId>>>,
    refetch_signal: Option<Signal<()>>,
) -> LocalResource<Result<Option<ViperTestSuiteParameters>, ErrorText>> {
    let id = id.into();
    let carl = use_app_globals().client;

    LocalResource::new(move || {
        let id = id.get();
        refetch_signal.track();

        let mut carl = carl.clone();
        async move {
            let Some(id) = id else {
                return Ok(None);
            };

            carl.viper.get_viper_test_suite_parameters(id).await.map(Some).map_err(|error| {
                error!("Unable to load VIPER parameters: {error}");
                let mut text = ErrorText {
                    headline: String::from("Unable to load VIPER parameters"),
                    description: client_error_description(
                        &error,
                        "The VIPER test suite parameters could not be loaded. Please check your connection and try again.",
                    ),
                };
                if let ClientError::UsageError(GetViperTestSuiteParametersError::Compilation { kind, .. }) = &error {
                    match kind {
                        CompilationErrorKind::FailedLoading { message, .. }
                        | CompilationErrorKind::InvalidSource { message }
                        | CompilationErrorKind::FailedInspection { message }
                        | CompilationErrorKind::PythonCompilationError { message }
                        | CompilationErrorKind::Internal { message } => {
                            text.description = format!("{}: \n {message}", text.description);
                        }
                        CompilationErrorKind::NoSuitableSourceLoader { .. } => {}
                    }
                }
                text
            })
        }
    })
}
