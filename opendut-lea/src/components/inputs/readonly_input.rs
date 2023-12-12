use leptos::*;

#[component]
pub fn ReadOnlyInput(
    #[prop(into)] label: MaybeSignal<String>,
    #[prop(into)] value: MaybeSignal<String>,
) -> impl IntoView {

    let aria_label = Clone::clone(&label);

    view! {
        <div class="field">
            <label class="label">{ label }</label>
            <div class="control">
                <input
                    class="input is-small is-static is-family-monospace is-clickable"
                    aria-label=move || aria_label.get()
                    type="text"
                    prop:value=move || value.get()
                    readonly
                />
            </div>
        </div>
    }
}
