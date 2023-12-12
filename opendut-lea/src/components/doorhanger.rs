use leptos::*;

pub type Trigger = Box<dyn FnOnce() -> View>;

#[allow(dead_code)]
pub enum DoorhangerAlignment {
    Left,
    Right,
}

#[component]
pub fn Doorhanger(
    #[prop(into)] visible: MaybeSignal<bool>,
    #[prop(into)] alignment: MaybeSignal<DoorhangerAlignment>,
    trigger: Trigger,
    children: Children
) -> impl IntoView {

    let doorhanger_classes = move || {
        alignment.with(|alignment| match alignment {
            DoorhangerAlignment::Left => "doorhanger is-right",
            DoorhangerAlignment::Right => "doorhanger is-left",
        })
    };

    view! {
        <div class={ doorhanger_classes } class=("is-active", move || visible.get())>
            <div class="doorhanger-trigger">
                { trigger() }
            </div>
            <div class="doorhanger-container">
                <div class="doorhanger-content p-2">
                    { children() }
                </div>
            </div>
        </div>
    }
}
