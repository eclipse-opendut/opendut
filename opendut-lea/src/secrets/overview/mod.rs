mod row;

use leptos::prelude::*;
use opendut_lea_components::{BasePageContainer, Breadcrumb, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, OverviewTable, TableHeading};
use opendut_model::secret::SecretDescriptor;
use crate::app::use_app_globals;
use crate::secrets::components::CreateSecretButton;
use crate::secrets::overview::row::Row;

#[component(transparent)]
pub fn SecretsOverview() -> impl IntoView {

    let globals = use_app_globals();

    let refetch_secrets = RwSignal::new(());

    let secrets: LocalResource<Vec<SecretDescriptor>> = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            refetch_secrets.track();

            let mut carl = carl.clone();

            async move {
                let mut secrets = carl.secret.list_secret_descriptors().await
                    .expect("Failed to request the list of secrets");

                secrets.sort_by(|secret_a, secret_b| {
                    secret_a.name.value().to_lowercase()
                        .cmp(&secret_b.name.value().to_lowercase())
                });

                secrets
            }
        })
    };

    let breadcrumbs = vec![
        Breadcrumb::new("Dashboard", "/"),
        Breadcrumb::new("Secrets", "/secrets")
    ];

    let table_headings = vec![
        TableHeading::new(String::from("Name")),
        TableHeading::new(String::from("Type")),
        TableHeading::new(String::from("Action")).set_narrow(),
    ];

    view! {
        <BasePageContainer
            title="Secrets"
            breadcrumbs
            controls=view! {
                <div class="buttons">
                    <CreateSecretButton />
                    <IconButton
                        icon=FontAwesomeIcon::ArrowsRotate
                        color=ButtonColor::Light
                        size=ButtonSize::Normal
                        state=ButtonState::Enabled
                        label="Refresh table of secrets"
                        on_action=move || {
                            refetch_secrets.notify();
                        }
                    />
                </div>
            }
        >
            <OverviewTable headings=table_headings>
                { move || Suspend::new(async move {
                    let secrets = secrets.await;
                    if secrets.is_empty() {
                        view! {
                            <tr>
                                <td colspan="3" class="has-text-centered">
                                    <p class="py-4 has-text-grey">"No secrets have been created yet."</p>
                                </td>
                            </tr>
                        }.into_any()
                    } else {
                        view! {
                            <For
                                each = move || secrets.clone()
                                key = |secret| secret.id
                                children = { move |secret_descriptor| {

                                    let on_delete = move || {
                                        refetch_secrets.notify();
                                    };

                                    view! {
                                        <Row
                                            secret_descriptor=RwSignal::new(secret_descriptor)
                                            on_delete
                                        />
                                    }
                                }}
                            />
                        }.into_any()
                    }
                })}
            </OverviewTable>
        </BasePageContainer>
    }
}
