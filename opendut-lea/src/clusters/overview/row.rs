use crate::clusters::IsDeployed;
use crate::clusters::components::{ClusterHealth, DeleteClusterButton, DeployToggle};
use leptos::prelude::*;
use opendut_lea_components::{ButtonColor, OverviewTableCell};
use opendut_model::cluster::ClusterDescriptor;
use opendut_model::cluster::state::ClusterState;
use crate::components::ClickableOverviewTableRow;

#[component]
pub fn Row<OnDeploymentChanged, OnDeleteFn>(
    cluster_descriptor: RwSignal<ClusterDescriptor>,
    is_deployed: RwSignal<IsDeployed>,
    on_deployment_changed: OnDeploymentChanged,
    on_delete: OnDeleteFn,
) -> impl IntoView
where
    OnDeploymentChanged: Fn() + Clone + Send + 'static,
    OnDeleteFn: Fn() + Copy + Send + 'static,
{
    let cluster_id = create_read_slice(cluster_descriptor, |cluster_descriptor| {
        cluster_descriptor.id
    });

    let cluster_name = create_read_slice(cluster_descriptor, |cluster_descriptor| {
        Clone::clone(&cluster_descriptor.name).to_string()
    });

    let configurator_href = Signal::derive(move || format!("/clusters/{}/configure/general", cluster_id.get()));

    let cluster_state = RwSignal::new(ClusterState::default());

    view! {
        <ClickableOverviewTableRow configurator_href>
            <OverviewTableCell>
                <div on:click=|event| event.stop_propagation()>
                    <DeployToggle
                        cluster_id
                        is_deployed
                        on_deployment_changed
                    />
                </div>
            </OverviewTableCell>

            <OverviewTableCell>
                <ClusterHealth state=cluster_state />
            </OverviewTableCell>

            <OverviewTableCell>
                <a href=configurator_href> { cluster_name } </a>
            </OverviewTableCell>

            <OverviewTableCell>
                <div class="is-pulled-right">
                    <DeleteClusterButton
                        cluster_id
                        deployed_signal=is_deployed
                        button_color=ButtonColor::TextDanger
                        on_delete
                    />
                </div>
            </OverviewTableCell>
        </ClickableOverviewTableRow>
    }
}
