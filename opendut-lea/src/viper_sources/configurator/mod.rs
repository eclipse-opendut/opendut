mod types;
mod components;
mod tabs;

use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use opendut_lea_components::{BasePageContainer, Breadcrumb, LoadingSpinner, UserInputError, UserInputValue};
use opendut_lea_components::tabs::{Tab, Tabs};
use opendut_model::viper::ViperSourceId;
use crate::app::use_app_globals;
use crate::components::use_active_tab;
use crate::routing::{navigate_to, WellKnownRoutes};
use crate::viper_sources::configurator::components::Controls;
use crate::viper_sources::configurator::tabs::{GeneralTab, TabIdentifier};
use crate::viper_sources::configurator::types::UserViperSourceConfiguration;

#[component(transparent)]
pub fn ViperSourceConfigurator() -> impl IntoView {

    let globals = use_app_globals();
    let params = use_params_map();

    let (viper_source_configuration, viper_source_configuration_resource, is_valid_configuration) = {
        let viper_source_id = {
            let viper_source_id = params.with_untracked(|params| {
                params.get("id").and_then(|id| ViperSourceId::try_from(id.as_str()).ok())
            });
            match viper_source_id {
                None => {
                    let use_navigate = use_navigate();
                    navigate_to(WellKnownRoutes::ErrorPage {
                        title: String::from("Invalid ViperSourceId"),
                        text: String::from("Could not parse the provided value as ViperSourceId!"),
                        details: None,
                    }, use_navigate);
                    ViperSourceId::random()
                }
                Some(viper_source_id) => {
                    viper_source_id
                }
            }
        };

        let viper_source_configuration = RwSignal::new(
            UserViperSourceConfiguration {
                id: viper_source_id,
                name: UserInputValue::Left(UserInputError::from("Enter a valid viper source name.")),
                url: UserInputValue::Left(UserInputError::from("Enter a valid viper source url.")),
                is_new: true,
            }
        );

        let viper_source_configuration_resource = LocalResource::new(move || {
            let mut carl = globals.client.clone();
            async move {
                if let Ok(configuration) = carl.viper.get_viper_source_descriptor(viper_source_id).await {
                    viper_source_configuration.update(|user_configuration| {
                        user_configuration.name = UserInputValue::Right(configuration.name.value().to_owned());
                        user_configuration.url = UserInputValue::Right(configuration.url.to_string());
                    })
                }
            }
        });

        let is_valid_configuration = Memo::new(move |_| {
            viper_source_configuration.with(|source_configuration| {
                source_configuration.name.is_right()
                && source_configuration.url.is_right()
            })
        });

        (viper_source_configuration, viper_source_configuration_resource, is_valid_configuration)
    };

    let viper_source_id_string = create_read_slice(viper_source_configuration, |config| config.id.to_string());

    let breadcrumbs = Signal::derive(move || {
        let viper_source_id = viper_source_id_string.get();
        vec![
            Breadcrumb::new("Dashboard", "/"),
            Breadcrumb::new("Viper Sources", "viper_sources"),
            Breadcrumb::new(&viper_source_id, format!("{viper_source_id}/configure")),
        ]
    });

    let subtitle = Signal::derive(move || {
        if let UserInputValue::Right(name) = viper_source_configuration.get().name {
            name
        } else {
            String::new()
        }
    });

    let tabs = Signal::derive(move || {
        vec![
            Tab::from_title_and_href(
                String::from("General"),
                TabIdentifier::General.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !viper_source_configuration.read().is_valid())),
        ]
    });
    
    let active_tab = use_active_tab::<TabIdentifier>();
    
    view! {
        <BasePageContainer
            title="Configure Viper Source"
            subtitle=subtitle
            breadcrumbs=breadcrumbs
            controls=view! { <Controls configuration=viper_source_configuration is_valid_configuration /> }
        >
            <Suspense
                fallback=move || view! { <LoadingSpinner /> }
            >
                {
                    move || Suspend::new(async move {
                        viper_source_configuration_resource.await;

                        view! {
                            <Tabs tabs active_tab=Signal::derive(move || active_tab.get().as_str())>
                                { move || match active_tab.get() {
                                    TabIdentifier::General => view! { <GeneralTab viper_source_configuration /> }
                                }}
                            </Tabs>
                        }
                    })
                }
            </Suspense>
        </BasePageContainer>
    }
}
