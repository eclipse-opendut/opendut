use leptos::*;
use leptos::html::Div;
use leptos_use::on_click_outside;

use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId};

use crate::app::{ExpectGlobals, use_app_globals};
use crate::clusters::components::CreateClusterButton;
use crate::components::{BasePageContainer, Breadcrumb, ButtonColor, ButtonState, FontAwesomeIcon, IconButton, Initialized};

#[component(transparent)]
pub fn ClustersOverview() -> impl IntoView {

    #[component]
    fn inner() -> impl IntoView {

        let globals = use_app_globals();

        let clusters = create_local_resource(|| {}, move |_| {
            let mut carl = globals.expect_client();
            async move {
                carl.cluster.list_cluster_configurations().await
                    .expect("Failed to request the list of clusters")
            }
        });

        let deploy_cluster = create_action(move |id: &ClusterId| {
            let mut carl = globals.expect_client();
            let id = Clone::clone(id);
            async move {
                let _ = carl.cluster.store_cluster_deployment(&ClusterDeployment { id }).await;
            }
        });

        let rows = move || {
            match clusters.get() {
                Some(configurations) => {
                    configurations.iter().cloned().map(|configuration| {
                        let cluster_id = configuration.id;
                        view! {
                            <Row
                                configuration
                                on_deploy=move || deploy_cluster.dispatch(cluster_id)
                            />
                        }
                    }).collect::<Vec<_>>()
                }
                None => {
                    Vec::new()
                }
            }
        };

        let breadcrumbs = vec![
            Breadcrumb::new("Dashboard", "/"),
            Breadcrumb::new("Clusters", "/clusters")
        ];

        view! {
            <BasePageContainer
                title="Clusters"
                breadcrumbs=breadcrumbs
                controls=view! {
                    <CreateClusterButton />
                }
            >
                <table class="table is-hoverable is-fullwidth">
                    <thead>
                        <tr>
                            <th class="is-narrow">"Health"</th>
                            <th>"ID"</th>
                            <th class="is-narrow">"Action"</th>
                        </tr>
                    </thead>
                    <tbody>
                        { rows }
                    </tbody>
                </table>
            </BasePageContainer>
        }
    }

    view! {
        <Initialized>
            <Inner />
        </Initialized>
    }
}

#[component]
fn Row<OnDeploy>(
    configuration: ClusterConfiguration,
    on_deploy: OnDeploy,
) -> impl IntoView
where OnDeploy: Fn() + 'static {
    let id = configuration.id.to_string();
    let href = {
        let id = Clone::clone(&id);
        move || { format!("/clusters/{}/configure/general", id) }
    };

    let dropdown_active = create_rw_signal(false);
    let dropdown = create_node_ref::<Div>();

    let _ = on_click_outside(dropdown, move |_| dropdown_active.set(false) );

    view! {
        <tr>
            <td class="is-vcentered">
                // health
            </td>
            <td class="is-vcentered">
                <a href={ href() } >{ Clone::clone(&id) }</a>
            </td>
            <td class="is-vcentered">
                <div class="is-pulled-right">
                    <div class="dropdown is-right" class=("is-active", move || dropdown_active.get())>
                        <div class="dropdown-trigger">
                            <IconButton
                                icon=FontAwesomeIcon::EllipsisVertical
                                color=ButtonColor::White
                                state=ButtonState::Enabled
                                label="Show Cluster Action Menu"
                                on_action=move || {
                                    dropdown_active.update(|value| *value = !*value);
                                }
                            />
                        </div>
                        <div node_ref=dropdown class="dropdown-menu">
                            <div class="dropdown-content">
                                <button
                                    class="button is-white is-fullwidth is-justify-content-flex-start"
                                    aria-label="Deploy Cluster"
                                    on:click=move |_| {
                                        dropdown_active.set(false);
                                        on_deploy();
                                    }
                                >
                                    <span class="icon">
                                        <i class="fa-solid fa-play"></i>
                                    </span>
                                    <span>"Deploy"</span>
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </td>
        </tr>
    }
}
