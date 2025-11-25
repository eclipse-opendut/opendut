use leptos::html::Div;
use leptos::prelude::*;
use opendut_lea_components::{health, ButtonColor, Toggle};
use opendut_lea_components::health::Health;
use opendut_model::cluster::ClusterDescriptor;
use crate::clusters::components::DeleteClusterButton;
use crate::clusters::IsDeployed;

#[component]
pub fn Row<OnDeployFn, OnUndeployFn, OnDeleteFn>(
    cluster_descriptor: RwSignal<ClusterDescriptor>,
    on_deploy: OnDeployFn,
    on_undeploy: OnUndeployFn,
    is_deployed: RwSignal<IsDeployed>,
    on_delete: OnDeleteFn,
) -> impl IntoView
where
    OnDeployFn: Fn() + 'static,
    OnUndeployFn: Fn() + 'static,
    OnDeleteFn: Fn() + Copy + Send + 'static,
{

    let cluster_id = create_read_slice(cluster_descriptor,
        |cluster_descriptor| {
            cluster_descriptor.id
        }
    );

    let cluster_name = create_read_slice(cluster_descriptor,
        |cluster_descriptor| {
            Clone::clone(&cluster_descriptor.name).to_string()
        }
    );

    let configurator_href = move || format!("/clusters/{}/configure/general", cluster_id.get());

    let dropdown_active = RwSignal::new(false);
    let dropdown = NodeRef::<Div>::new();

    let _ = leptos_use::on_click_outside(dropdown, move |_| dropdown_active.set(false));

    let health_state = Signal::derive(move || {
        health::State { //TODO implement Cluster health in backend and display it here
            kind: health::StateKind::Unknown,
            text: String::from("Unknown"),
        }
    });

    view! {
        <tr>
            <td class="is-vcentered has-text-centered">
                <Toggle
                    is_active = Signal::derive(move || {
                        is_deployed.get().0
                    })
                    on_action = move || {
                        if is_deployed.get().0 { on_undeploy() } else { on_deploy() }
                    }
                />
            </td>
            <td class="is-vcentered has-text-centered">
                <Health state=health_state />
            </td>
            <td class="is-vcentered">
                <a href=configurator_href> { cluster_name } </a>
            </td>
            <td class="is-vcentered is-flex is-justify-content-center">
                <div class="is-pulled-right">
                    <DeleteClusterButton
                        cluster_id
                        deployed_signal=is_deployed
                        button_color=ButtonColor::TextDanger
                        on_delete
                    />
                </div>
            </td>
        </tr>
    }
}
