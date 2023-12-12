use leptos::*;


#[component]
pub fn Tooltip(
    visible: ReadSignal<bool>,
    children: Children
) -> impl IntoView {
    view! {
        <div class="tooltip is-left" class=("is-active", move || visible.get())>
            <div class="tooltip-container">
                <div class="tooltip-content p-1 is-size-7 has-text-weight-medium">
                    { children() }
                </div>
            </div>
        </div>
    }
}
