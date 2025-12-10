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
use crate::sources::configurator::components::Controls;
use crate::sources::configurator::tabs::{GeneralTab, TabIdentifier};
use crate::sources::configurator::types::UserSourceConfiguration;

#[component(transparent)]
pub fn SourceConfigurator() -> impl IntoView {

    let globals = use_app_globals();
    let params = use_params_map();

    let (source_configuration, source_configuration_resource, is_valid_source_configuration) = {
        let source_id = {
            let source_id = params.with_untracked(|params| {
                params.get("id").and_then(|id| ViperSourceId::try_from(id.as_str()).ok())
            });
            match source_id {
                None => {
                    let use_navigate = use_navigate();
                    navigate_to(WellKnownRoutes::ErrorPage {
                        title: String::from("Invalid SourceId"),
                        text: String::from("Could not parse the provided value as SourceId!"),
                        details: None,
                    }, use_navigate);
                    ViperSourceId::random()
                }
                Some(source_id) => {
                    source_id
                }
            }
        };

        let source_configuration = RwSignal::new(
            UserSourceConfiguration {
                id: source_id,
                name: UserInputValue::Left(UserInputError::from("Enter a valid source name.")),
                url: UserInputValue::Left(UserInputError::from("Enter a valid source url.")),
                is_new: true,
            }
        );

        let source_configuration_resource = LocalResource::new(move || {
            let mut carl = globals.client.clone();
            async move {
                if let Ok(configuration) = carl.viper.get_viper_source_descriptor(source_id).await {
                    source_configuration.update(|user_configuration| {
                        user_configuration.name = UserInputValue::Right(configuration.name.value().to_owned());
                        user_configuration.url = UserInputValue::Right(configuration.url.to_string());
                    })
                }
            }
        });

        let is_valid_source_configuration = Memo::new(move |_| {
            source_configuration.with(|source_configuration| {
                source_configuration.name.is_right()
                && source_configuration.url.is_right()
            })
        });

        (source_configuration, source_configuration_resource, is_valid_source_configuration)
    };

    let source_id_string = create_read_slice(source_configuration, |config| config.id.to_string());

    let breadcrumbs = Signal::derive(move || {
        let source_id = source_id_string.get();
        vec![
            Breadcrumb::new("Dashboard", "/"),
            Breadcrumb::new("Sources", "sources"),
            Breadcrumb::new(&source_id, format!("{source_id}/configure")),
        ]
    });

    let subtitle = Signal::derive(move || {
        if let UserInputValue::Right(name) = source_configuration.get().name {
            name
        } else {
            String::new()
        }
    });

    let tabs = Signal::derive(move || {
        vec![
            Tab {
                title: String::from("General"),
                href: TabIdentifier::General.as_str().to_owned(),
                is_error: Signal::derive(move || {
                    let config = source_configuration.get();
                    let has_valid_name = config.name.is_right();
                    let has_valid_url  = config.url.is_right();

                    !(has_valid_name && has_valid_url)
                })
            }
        ]
    });
    
    let active_tab = use_active_tab::<TabIdentifier>();
    
    view! {
        <BasePageContainer
            title="Configure Source"
            subtitle=subtitle
            breadcrumbs=breadcrumbs
            controls=view! { <Controls configuration=source_configuration is_valid_source_configuration=is_valid_source_configuration.into() /> }
        >
            <Suspense
                fallback=move || view! { <LoadingSpinner /> }
            >
                {
                    move || Suspend::new(async move {
                        source_configuration_resource.await;

                        view! {
                            <Tabs tabs active_tab=Signal::derive(move || active_tab.get().as_str())>
                                { move || match active_tab.get() {
                                    TabIdentifier::General => view! { <GeneralTab source_configuration /> }
                                }}
                            </Tabs>
                        }
                    })
                }
            </Suspense>
        </BasePageContainer>
    }
}
