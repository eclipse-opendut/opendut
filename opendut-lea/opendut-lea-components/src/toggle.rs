use leptos::prelude::*;

#[component]
pub fn Toggle<F>(
    #[prop(optional, into)] text: Option<Signal<String>>,
    is_active: Signal<bool>,
    on_action: F,
) -> impl IntoView
where F: Fn() + 'static {

    view! {
        <div
            class="is-flex is-align-items-center is-justify-content-center"
            on:click=move |event| event.stop_propagation()
        >
            <label class="dut-toggle"
                class:active = move || is_active.get()
                on:click=move |_| on_action()
            />
            {
                text.map(|text| {
                    view! {
                        <span class="pl-2">{ text }</span>
                    }
                })
            }
        </div>
    }
}
