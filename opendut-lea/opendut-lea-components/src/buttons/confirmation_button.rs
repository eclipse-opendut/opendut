use leptos::prelude::*;
use leptos::html::Div;
use leptos_use::on_click_outside;
use crate::{ButtonSize, IconButton};
use crate::{ButtonColor, ButtonState, FontAwesomeIcon};
use crate::doorhanger::{Doorhanger, DoorhangerAlignment};

#[component]
pub fn ConfirmationButton<OnConfirm>(
    #[prop(into)] icon: Signal<FontAwesomeIcon>,
    #[prop(into)] color: Signal<ButtonColor>,
    #[prop(into)] size: Signal<ButtonSize>,
    #[prop(into)] state: Signal<ButtonState>,
    #[prop(into)] label: Signal<String>,
    on_confirm: OnConfirm,
) -> impl IntoView
where OnConfirm: Fn() + Send + 'static {

    let text = Clone::clone(&label);
    let aria_label = Clone::clone(&label);

    let doorhanger_visible = RwSignal::new(false);

    let delete_button_area = NodeRef::<Div>::new();

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
                    }.into_any()
                })
            >
                <div class="is-flex is-flex-direction-column p-1">
                    <p class="is-size-6 has-text-weight-semibold pb-2">{ text }</p>
                    <div class="columns is-gapless is-mobile m-0">
                        <div class="column mr-2">
                            <IconButton
                                icon=FontAwesomeIcon::Check
                                color=ButtonColor::Success
                                size=ButtonSize::Normal
                                state=ButtonState::Enabled
                                label="Yes"
                                show_label=true
                                is_outlined=true
                                on_action= move || {
                                    doorhanger_visible.set(false);
                                    on_confirm();
                                }
                            />
                        </div>
                        <div class="column">
                            <IconButton
                                icon=FontAwesomeIcon::XMark
                                color=ButtonColor::Danger
                                size=ButtonSize::Normal
                                state=ButtonState::Enabled
                                label="No"
                                show_label=true
                                is_outlined=true
                                on_action= move || {
                                    doorhanger_visible.set(false);
                                }
                            />
                        </div>
                    </div>
                </div>
            </Doorhanger>
        </div>
    }
}
