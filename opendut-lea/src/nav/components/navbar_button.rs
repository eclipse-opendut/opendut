use leptos::prelude::*;
use leptos_router::hooks::use_location;
use opendut_lea_components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton};

#[component]
pub fn NavbarButton(
    #[prop(into)] icon: Signal<FontAwesomeIcon>,
    #[prop(into)] label: Signal<String>,
    #[prop(into)] path: Signal<String>,
    #[prop(default=true)] show_label: bool,
    #[prop(default=false)] external_link: bool,
) -> impl IntoView {
    let location = use_location();
    let is_active = move || location.pathname.get() == path.get();

    let (rel, target) = if external_link {
        (Some("external"), Some("_blank"))
    } else {
        (None, None)
    };

    view! {
        <a
            class="navbar-item px-0 mx-2"
            class=(["is-active", "is-tab"], is_active)
            href=path.get()
            rel=rel
            target=target
        >
            <IconButton
                icon
                color=ButtonColor::Light
                size=ButtonSize::Normal
                state=ButtonState::Enabled
                label
                show_label
                skip_stop_propagation=true
                on_action=||{}
            />
        </a>
    }
}
