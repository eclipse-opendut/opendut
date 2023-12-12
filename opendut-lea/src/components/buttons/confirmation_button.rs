use leptos::*;
use leptos::html::Div;
use leptos_use::on_click_outside;
use crate::components::IconButton;
use crate::components::{ButtonColor, ButtonState, FontAwesomeIcon};
use crate::components::doorhanger::{Doorhanger, DoorhangerAlignment};

#[component]
pub fn ConfirmationButton<A>(
    #[prop(into)] icon: MaybeSignal<FontAwesomeIcon>,
    #[prop(into)] color: MaybeSignal<ButtonColor>,
    #[prop(into)] state: MaybeSignal<ButtonState>,
    #[prop(into)] label: MaybeSignal<String>,
    on_conform: A,
) -> impl IntoView
where A: Fn() + 'static {

    let text = Clone::clone(&label);
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
                            state=state
                            label=aria_label
                            on_action=move || doorhanger_visible.set(true)
                        />
                    }
                })
            >
                <div class="columns">
                    <div class="column">
                        <p class="is-size-6 has-text-weight-semibold mb-2">{ text }</p>
                        <div class="columns is-gapless is-mobile">
                            <div class="column mr-2">
                                <button
                                    class="button is-success is-outlined mb-0"
                                    aria-label=move || label.get()
                                    on:click=move |_| {
                                        doorhanger_visible.set(false);
                                        on_conform();
                                    }
                                >
                                    <span class="icon">
                                        <i class="fa-solid fa-check"></i>
                                    </span>
                                    <span>"Yes"</span>
                                </button>
                            </div>
                            <div class="column">
                                <button
                                    class="button is-danger is-outlined mb-0"
                                    aria-label="Cancel Delete Cluster"
                                    on:click=move |_| doorhanger_visible.update(|visible| *visible = !*visible)
                                >
                                    <span class="icon">
                                        <i class="fa-solid fa-xmark"></i>
                                    </span>
                                    <span>"No"</span>
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </Doorhanger>
        </div>
    }
}
