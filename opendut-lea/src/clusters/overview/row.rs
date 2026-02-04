use crate::clusters::IsDeployed;
use crate::clusters::components::DeleteClusterButton;
use leptos::html::Div;
use leptos::prelude::*;
use leptos::web_sys;
use leptos_router::hooks::use_navigate;
use opendut_lea_components::tooltip::Tooltip;
use opendut_lea_components::{ButtonColor, Toggle};
use opendut_model::cluster::ClusterDescriptor;
use crate::clusters::components::ClusterHealth;
use opendut_model::cluster::state::ClusterState;

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

    let configurator_href = move || format!("/clusters/{}/configure/general", cluster_id.get());

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

    let mousedown_pos = RwSignal::new((0, 0));
    let use_navigate = use_navigate();

    let on_mousedown = move |e: web_sys::MouseEvent| {
        mousedown_pos.set((e.client_x(), e.client_y()));
    };

    let on_click = move |e: web_sys::MouseEvent| {
        let distance = crate::util::calculate_distance(mousedown_pos.get(), (e.client_x(), e.client_y()));
        // fixes text selection issue: mouse moved < threshold -> click, else it's a drag.
        if distance < crate::util::MOUSE_DRAG_PIXEL_THRESHOLD {
            use_navigate(&configurator_href(), Default::default());
        }
    };

    let cluster_state = RwSignal::new(ClusterState::default());

    view! {
        <tr
            class="is-clickable"
            on:mousedown=on_mousedown
            on:click=on_click
        >
            <td class="is-vcentered has-text-centered">
                <Tooltip
                    text=tooltip_text
                >
                    <div on:click=|e| e.stop_propagation() on:mousedown=|e| e.stop_propagation()>
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
            </td>
            <td class="is-vcentered has-text-centered">
                <ClusterHealth state=cluster_state />
            </td>
            <td class="is-vcentered">
                <a href=configurator_href on:click=|e| e.stop_propagation() on:mousedown=|e| e.stop_propagation()> { cluster_name } </a>
            </td>
            <td class="is-vcentered is-flex is-justify-content-center">
                <div class="is-pulled-right" on:click=|e| e.stop_propagation() on:mousedown=|e| e.stop_propagation()>
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
