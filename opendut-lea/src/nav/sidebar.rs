use crate::routing;
use leptos::prelude::*;
use leptos_router::hooks::use_location;
use opendut_lea_components::FontAwesomeIcon;

#[component]
pub fn Sidebar(menu_visible: RwSignal<bool>, hide_buttons: Signal<bool>) -> impl IntoView {
    view! {
        <aside class="dut-menu is-left column" class=("is-active", move || menu_visible.get())>
            <ul class="dut-menu-list" class=("is-hidden", move || hide_buttons.get())>
                <SidebarItem
                    icon= FontAwesomeIcon::Dashboard
                    label="Dashboard"
                    path=routing::path::dashboard
                />
                <SidebarItem
                    icon= FontAwesomeIcon::Cluster
                    label="Clusters"
                    path=routing::path::clusters_overview
                />
                <SidebarItem
                    icon= FontAwesomeIcon::Peers
                    label="Peers"
                    path=routing::path::peers_overview
                />
                {
                    #[cfg(feature = "viper")]
                    view!{
                        <SidebarItem
                            icon= FontAwesomeIcon::Link
                            label="Sources"
                            path=routing::path::sources_overview
                        />
                    }
                }
                {
                    #[cfg(feature = "viper")]
                    view!{
                        <SidebarItem
                            icon= FontAwesomeIcon::Code
                            label="Tests"
                            path=routing::path::tests_overview
                        />
                    }
                }
                <SidebarItem
                    icon= FontAwesomeIcon::Downloads
                    label="Downloads"
                    path=routing::path::downloads
                />
            </ul>
            <div class="px-2">
                <div class="dut-divider" class=("is-hidden", move || hide_buttons.get()) />
                <ul class="menu-label">
                    <li class=("is-hidden", move || hide_buttons.get())>
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
    #[prop(into)] path: Signal<String>,
) -> impl IntoView {

    let location = use_location();
    let is_active = move || location.pathname.get() == path.get();

    view! {
        <li>
            <a class=("is-active", is_active) href=path.get()>
                <i class=icon.get().as_class() />
                <span class="ml-2 is-size-6"> {label} </span>
            </a>
        </li>
    }
}
