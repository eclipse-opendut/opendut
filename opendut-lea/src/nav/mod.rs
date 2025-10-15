pub mod sidebar;
mod profile_dropdown;

use leptos::html::Div;
use leptos::prelude::*;
use leptos_use::on_click_outside;
use crate::components::{ ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton};
use crate::routing;
use crate::components::navbar_button::NavbarButton;
use crate::nav::profile_dropdown::ProfileDropdown;
use crate::routing::WellKnownRoutes;

#[component]
pub fn Navbar(
    menu_visible: RwSignal<bool>,
    hide_buttons: Signal<bool>,
) -> impl IntoView {

    let menu_button_icon = Signal::derive(move || {
        if menu_visible.get() {
            FontAwesomeIcon::XMark
        } else {
            FontAwesomeIcon::Bars
        }
    });

    let menu_button_area = NodeRef::<Div>::new();
    let _ = on_click_outside(menu_button_area, move |_| {
        menu_visible.set(false)
    });

    view! {
        <nav class="dut-navbar columns is-vcentered is-mobile pr-3 m-0">
            <div class="column is-narrow" node_ref=menu_button_area>
                <IconButton
                    icon=menu_button_icon
                    color=ButtonColor::Light
                    size=ButtonSize::Normal
                    state=ButtonState::Enabled
                    label="User"
                    on_action=move || menu_visible.update(|is_visible| *is_visible = !*is_visible)
                />
            </div>
            <div class="navbar-brand dut-logo">
                <a class="navbar-item" href=routing::path::dashboard>
                    <img src="/logos/logo_onlight_wide.svg" alt="openDuT" />
                </a>
            </div>
            <div class="navbar-menu" class:is-hidden=move || hide_buttons.get()>
                <div class="navbar-end">
                    <NavbarButton
                        icon=FontAwesomeIcon::Cluster
                        label="Clusters"
                        route=WellKnownRoutes::ClustersOverview
                        path=routing::path::clusters_overview
                    />
                    <NavbarButton
                        icon=FontAwesomeIcon::Peers
                        label="Peers"
                        route=WellKnownRoutes::PeersOverview
                        path=routing::path::peers_overview
                    />
                    <NavbarButton
                        icon=FontAwesomeIcon::Downloads
                        label="Downloads"
                        route=WellKnownRoutes::Downloads
                        path=routing::path::downloads
                    />
                </div>
            </div>
            <div class="column is-narrow ml-auto p-1" class:is-hidden=move || hide_buttons.get()>
                <ProfileDropdown />
            </div>
        </nav>
    }
}
