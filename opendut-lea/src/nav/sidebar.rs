use leptos::prelude::*;
use leptos_router::hooks::use_location;
use opendut_lea_components::FontAwesomeIcon;
use crate::routing;

#[component]
pub fn Sidebar(menu_visible: RwSignal<bool>, hide_buttons: Signal<bool>) -> impl IntoView {

    view! {
        <aside class="dut-menu is-left column" class:is-active=move || menu_visible.get()>
            <ul class="dut-menu-list" class:is-hidden= move || hide_buttons.get()>
                <SidebarItem
                    icon= FontAwesomeIcon::Dashboard
                    label="Dashboard"
                    route=routing::path::dashboard
                />
                <SidebarItem
                    icon= FontAwesomeIcon::Cluster
                    label="Clusters"
                    route=routing::path::clusters_overview
                />
                <SidebarItem
                    icon= FontAwesomeIcon::Peers
                    label="Peers"
                    route=routing::path::peers_overview
                />
                {
                    #[cfg(feature = "viper")]
                    view!{
                        <SidebarItem
                            icon= FontAwesomeIcon::Link
                            label="Sources"
                            route=routing::path::sources_overview
                        />
                    }
                }
                {
                    #[cfg(feature = "viper")]
                    view!{
                        <SidebarItem
                            icon= FontAwesomeIcon::Code
                            label="Tests"
                            route=routing::path::tests_overview
                        />
                    }
                }
                <SidebarItem
                    icon= FontAwesomeIcon::Downloads
                    label="Downloads"
                    route=routing::path::downloads
                />
            </ul>
            <div class="px-2">
                <div class="dut-divider" class:is-hidden= move || hide_buttons.get()/>
                <ul class="menu-label">
                    <li class:is-hidden= move || hide_buttons.get()>
                        <a href=routing::path::about> About </a>
                    </li>
                    <li>
                        <a href=routing::path::licenses> Licenses </a>
                    </li>
                </ul>
            </div>
        </aside>
    }
}

#[component]
pub fn SidebarItem(
    #[prop(into)] icon: Signal<FontAwesomeIcon>,
    #[prop(into)] label: Signal<String>,
    #[prop(into)] route: Signal<String>,
) -> impl IntoView {

    let location = use_location();
    let is_active = move |path: &str| location.pathname.get() == path;

    view! {
        <li>
            <a class:is-active= move || is_active(route.read().as_str()) href=route.get()>
                <i class=icon.get().as_class() />
                <span class="ml-2 is-size-6"> {label} </span>
            </a>
        </li>
    }
}
