use leptos::prelude::*;
use leptos::html::Div;
use leptos_use::{use_element_size, UseElementSizeReturn};

pub type Trigger = Box<dyn FnOnce() -> AnyView + Send>;

#[allow(dead_code)]
pub enum DoorhangerAlignment {
    Left,
    Right,
}

#[allow(dead_code)]
pub enum DoorhangerTriggerSize {
    Small,
    Normal,
    Medium,
    Large,
}

#[component]
pub fn Doorhanger(
    #[prop(into)] visible: Signal<bool>,
    #[prop(into)] alignment: Signal<DoorhangerAlignment>,
    trigger: Trigger,
    children: Children
) -> impl IntoView {

    let doorhanger_classes = move || {
        alignment.with(|alignment| match alignment {
            DoorhangerAlignment::Left => "doorhanger is-right",
            DoorhangerAlignment::Right => "doorhanger is-left",
        })
    };

    let trigger_div = NodeRef::<Div>::new();
    let UseElementSizeReturn { height, .. } = use_element_size(trigger_div);
    let dog_ear_style = move || {
        let top = (height.get() as i32) + 4;
        format!("top: {top}px")
    };

    view! {
        <div class={ doorhanger_classes } class=("is-active", move || visible.get())>
            <div node_ref=trigger_div class="doorhanger-trigger">
                <div>{ trigger() }</div>
                <div class="doorhanger-dog-ear" style=dog_ear_style></div>
            </div>
            <div class="doorhanger-container">
                <div class="doorhanger-content p-2">
                    { children() }
                </div>
            </div>
        </div>
    }
}
