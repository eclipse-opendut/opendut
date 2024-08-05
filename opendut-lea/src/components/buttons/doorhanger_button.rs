use leptos::*;
use leptos::html::Div;
use leptos_use::on_click_outside;
use crate::components::{ButtonSize, IconButton};
use crate::components::{ButtonColor, ButtonState, FontAwesomeIcon};
use crate::components::doorhanger::{Doorhanger, DoorhangerAlignment};

#[component]
pub fn DoorhangerButton(
    #[prop(into)] icon: MaybeSignal<FontAwesomeIcon>,
    #[prop(into)] color: MaybeSignal<ButtonColor>,
    #[prop(into)] size: MaybeSignal<ButtonSize>,
    #[prop(into)] state: MaybeSignal<ButtonState>,
    #[prop(into)] label: MaybeSignal<String>,
    #[prop(into)] text: MaybeSignal<HtmlElement<Div>>,
) -> impl IntoView {

    let text = Clone::clone(&text);
    let aria_label = Clone::clone(&label);

    let doorhanger_visible = create_rw_signal(false);

    let delete_button_area = create_node_ref::<Div>();

    let _ = on_click_outside(delete_button_area, move |_| {
        doorhanger_visible.set(false)
    });

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
                    }
                })
            >
                <p class="is-size-6 has-text-weight-semibold">{ text }</p>
            </Doorhanger>
        </div>
    }
}
