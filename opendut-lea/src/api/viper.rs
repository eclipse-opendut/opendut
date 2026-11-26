use leptos::prelude::{LocalResource, RwSignal, Track};
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
