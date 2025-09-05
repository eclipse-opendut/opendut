use leptos::prelude::*;
use leptos::html::Div;
use leptos_use::on_click_outside;
use crate::components::{ButtonSize, IconButton};
use crate::components::{ButtonColor, ButtonState, FontAwesomeIcon};
use crate::components::doorhanger::{Doorhanger, DoorhangerAlignment};

#[component]
pub fn DoorhangerButton(
    #[prop(into)] icon: Signal<FontAwesomeIcon>,
    #[prop(into)] color: Signal<ButtonColor>,
    #[prop(into)] size: Signal<ButtonSize>,
    #[prop(into)] state: Signal<ButtonState>,
    #[prop(into)] label: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {

    let aria_label = Clone::clone(&label);

    let doorhanger_visible = RwSignal::new(false);

    let delete_button_area = NodeRef::<Div>::new();

    let _ = on_click_outside(delete_button_area, move |_| {
        doorhanger_visible.set(false)
    });

    let children = StoredValue::new(children);

    view! {
        <div node_ref=delete_button_area>
            <Doorhanger
                visible=doorhanger_visible.read_only()
                alignment=DoorhangerAlignment::Left
                trigger=Box::new(move || {
                    view! {
                        <IconButton
                            icon=icon
                            color=color
                            size=size
                            state=state
                            label=aria_label
                            on_action=move || doorhanger_visible.set(true)
                        />
                    }.into_any()
                })
            >
                <p class="is-size-6 has-text-weight-semibold">{ children.read_value()() }</p>
            </Doorhanger>
        </div>
    }
}
