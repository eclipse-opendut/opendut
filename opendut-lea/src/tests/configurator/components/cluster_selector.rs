use leptos::prelude::*;
use opendut_lea_components::{UserInputValue, NON_BREAKING_SPACE};
use opendut_model::cluster::ClusterId;
use crate::app::use_app_globals;
use crate::tests::configurator::types::UserTestConfiguration;

#[component]
pub fn ClusterSelector(test_configuration: RwSignal<UserTestConfiguration>) -> impl IntoView {

    let globals = use_app_globals();

    let registered_clusters = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            let mut carl = carl.clone();
            async move {
                carl.cluster.list_cluster_descriptors().await
                    .expect("Failed to request the list of clusters")
            }
        })
    };

    let (getter, setter) = create_slice(test_configuration,
        |config| {
            Clone::clone(&config.cluster)
        },
        |config, input| {
            config.cluster = input;
        }
    );

    let help_text = move || {
        getter.with(|selection| match selection {
            UserInputValue::Left(error) => error.to_owned(),
            UserInputValue::Right(_) => String::from(NON_BREAKING_SPACE),
            UserInputValue::Both(error, _) => error.to_owned(),
        })
    };

    let clusters = Signal::derive(move || {
        if let Some(mut clusters) = registered_clusters.get() {
            clusters.sort_by(|cluster_a, cluster_b| {
                cluster_a.name.value().to_lowercase()
                    .cmp(&cluster_b.name.value().to_lowercase())
            });

            if clusters.is_empty() {
                setter.set(UserInputValue::Left(String::from("No clusters available.")));
            } else if matches!(getter.get(), UserInputValue::Left(_)) {
                setter.set(UserInputValue::Left(String::from("Select a cluster.")));
            }

            clusters
        } else {
            Vec::new()
        }
    });

    let is_selected = move |cluster: ClusterId| {
        let cluster = Clone::clone(&cluster);
        Signal::derive(move || {
            let getter = getter.get();
            match getter {
                UserInputValue::Right(selected) => cluster.to_string() == selected,
                UserInputValue::Left(_) | UserInputValue::Both(_, _) => false,
            }
        })
    };

    view! {
        <p class="help has-text-danger"> { help_text } </p>
        <div class="table-container mt-2">
            <table class="table is-fullwidth">
                <thead>
                    <tr>
                        <th></th>
                        <th>Name</th>
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || clusters.get()
                        key=|cluster| cluster.id
                        children=move |cluster| {
                            let cluster_id = cluster.id;
                            let cluster_href = move || format!("/cluster/{}/configure/general", cluster_id);

                            let is_selected = is_selected(Clone::clone(&cluster_id));

                            view! {
                                <tr
                                    class:has-background-link-light=move || is_selected.get()
                                    style="cursor: pointer;"
                                    on:click=move |_| {
                                        setter.set(UserInputValue::Right(cluster_id.to_string()));
                                    }
                                >
                                    <td class="is-narrow">
                                        <div class="control">
                                            <label class="radio">
                                                <input
                                                    type="radio"
                                                    name="selected-cluster"
                                                    prop:checked=is_selected
                                                    on:click=move |_| {
                                                        setter.set(UserInputValue::Right(cluster_id.to_string()));
                                                    }
                                                />
                                            </label>
                                        </div>
                                    </td>
                                    <td>
                                        <a href=cluster_href>
                                            { cluster.name.to_string() }
                                        </a>
                                    </td>
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        </div>
    }
}
