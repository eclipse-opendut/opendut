use std::sync::Arc;
use opendut_carl_api::carl::ClientError;
use opendut_carl_api::carl::viper::{GetViperSourceDescriptorError, GetViperTestRunDescriptorError, GetViperTestSuiteParametersError};
use opendut_model::viper::{ViperSourceId, ViperTestId, ViperTestSuiteParameters};
use leptos::prelude::{Get, LocalResource, RwSignal, Signal, Track};
use opendut_lea_components::ErrorText;
use opendut_model::viper::{ViperSourceDescriptor, ViperTestRunDescriptor};
use tracing::error;

use crate::app::use_app_globals;
use crate::util::client_error_description;

pub fn list_viper_source_descriptors(
    refetch_signal: Option<RwSignal<()>>,
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
    refetch_signal: Option<RwSignal<()>>,
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
    refetch_signal: Option<RwSignal<()>>,
) -> LocalResource<Result<ViperSourceDescriptor, Arc<ClientError<GetViperSourceDescriptorError>>>> {
    let id = id.into();
    let carl = use_app_globals().client;

    LocalResource::new(move || {
        let id = id.get();
        refetch_signal.track();

        let mut carl = carl.clone();
        async move {
            carl.viper.get_viper_source_descriptor(id).await.map_err(Arc::new)
        }
    })
}

pub fn get_viper_test_run_descriptor(
    id: impl Into<Signal<ViperTestId>>,
    refetch_signal: Option<RwSignal<()>>,
) -> LocalResource<Result<ViperTestRunDescriptor, Arc<ClientError<GetViperTestRunDescriptorError>>>> {
    let id = id.into();
    let carl = use_app_globals().client;

    LocalResource::new(move || {
        let id = id.get();
        refetch_signal.track();

        let mut carl = carl.clone();
        async move {
            carl.viper.get_viper_test_run_descriptor(id).await.map_err(Arc::new)
        }
    })
}

pub fn get_viper_test_suite_parameters(
    id: impl Into<Signal<ViperSourceId>>,
    refetch_signal: Option<RwSignal<()>>,
) -> LocalResource<Result<ViperTestSuiteParameters, Arc<ClientError<GetViperTestSuiteParametersError>>>> {
    let id = id.into();
    let carl = use_app_globals().client;

    LocalResource::new(move || {
        let id = id.get();
        refetch_signal.track();

        let mut carl = carl.clone();
        async move {
            carl.viper.get_viper_test_suite_parameters(id).await.map_err(Arc::new)
        }
    })
}
