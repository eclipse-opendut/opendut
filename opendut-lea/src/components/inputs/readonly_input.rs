use leptos::prelude::*;

#[component]
pub fn ReadOnlyInput(
    #[prop(into)] label: Signal<String>,
    #[prop(into)] value: Signal<String>,
) -> impl IntoView {

    let aria_label = Clone::clone(&label);

    view! {
        <div class="field">
            <label class="label">{ label }</label>
            <div class="control">
                <span
                    class="is-family-monospace is-clickable"
                    aria-label=move || aria_label.get()
                >{value}</span>
            </div>
        </div>
    }
}
