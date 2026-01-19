use crate::clusters::IsDeployed;
use crate::clusters::components::DeleteClusterButton;
use leptos::html::Div;
use leptos::prelude::*;
use leptos::web_sys;
use leptos_router::hooks::use_navigate;
use opendut_lea_components::health::Health;
use opendut_lea_components::tooltip::Tooltip;
use opendut_lea_components::{ButtonColor, Toggle, health};
use opendut_model::cluster::ClusterDescriptor;

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

    let health_state = Signal::derive(move || {
        health::State {
            //TODO implement Cluster health in backend and display it here
            kind: health::StateKind::Unknown,
            text: String::from("Unknown"),
        }
    });

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
        let (start_x, start_y) = mousedown_pos.get();
        let (end_x, end_y) = (e.client_x(), e.client_y());
        let diff_x = (end_x - start_x) as f64;
        let diff_y = (end_y - start_y) as f64;
        // euclidean distance formula -> distance = sqrt(diff_x² + diff_y²)
        let distance = (diff_x * diff_x + diff_y * diff_y).sqrt();
        if distance < 5.0 {
            use_navigate(&configurator_href(), Default::default());
        }
    };

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
                <Health state=health_state />
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
