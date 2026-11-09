use leptos::prelude::*;

#[component]
pub fn WarningMessage(#[prop(into)] text: String) -> impl IntoView {
    view! {
        <div class="notification is-warning">
            <div class="columns is-mobile is-vcentered">
                <div class="column is-narrow">
                    <i class="fa-solid fa-triangle-exclamation fa-2xl"></i>
                </div>
                <div class="column">
                    <p>{ text }</p>
                </div>
            </div>
        </div>
    }
}
