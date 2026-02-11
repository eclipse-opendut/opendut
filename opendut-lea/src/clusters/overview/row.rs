use crate::clusters::IsDeployed;
use crate::clusters::components::DeleteClusterButton;
use leptos::html::Div;
use leptos::prelude::*;
use opendut_lea_components::tooltip::Tooltip;
use opendut_lea_components::{ButtonColor, OverviewTableCell, Toggle};
use opendut_model::cluster::ClusterDescriptor;
use crate::clusters::components::ClusterHealth;
use opendut_model::cluster::state::ClusterState;
use crate::components::ClickableOverviewTableRow;

#[component]
pub fn Row<OnDeployFn, OnUndeployFn, OnDeleteFn>(
    cluster_descriptor: RwSignal<ClusterDescriptor>,
    on_deploy: OnDeployFn,
    on_undeploy: OnUndeployFn,
    is_deployed: RwSignal<IsDeployed>,
    on_delete: OnDeleteFn,
) -> impl IntoView
where
    OnDeployFn: Fn() + Send + 'static,
    OnUndeployFn: Fn() + Send + 'static,
    OnDeleteFn: Fn() + Copy + Send + 'static,
{
    let cluster_id = create_read_slice(cluster_descriptor, |cluster_descriptor| {
        cluster_descriptor.id
    });

    let cluster_name = create_read_slice(cluster_descriptor, |cluster_descriptor| {
        Clone::clone(&cluster_descriptor.name).to_string()
    });

    let configurator_href = Signal::derive(move || format!("/clusters/{}/configure/general", cluster_id.get()));

    let dropdown_active = RwSignal::new(false);
    let dropdown = NodeRef::<Div>::new();

    let _ = leptos_use::on_click_outside(dropdown, move |_| dropdown_active.set(false));

    let tooltip_text = Signal::derive(move || {
        if is_deployed.get().0 {
            "Deployment requested".to_string()
        } else {
            "Undeployed".to_string()
        }
    });

    let cluster_state = RwSignal::new(ClusterState::default());

    view! {
        <ClickableOverviewTableRow configurator_href>
            <OverviewTableCell>
                <Tooltip
                    text=tooltip_text
                >
                    <div>
                        <Toggle
                            is_active = Signal::derive(move || {
                                is_deployed.get().0
                            })
                            on_action = move || {
                                if is_deployed.get().0 { on_undeploy() } else { on_deploy() }
                            }
                        />
                    </div>
                </Tooltip>
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
