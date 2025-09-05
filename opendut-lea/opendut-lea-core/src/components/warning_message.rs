use leptos::prelude::*;

#[component]
pub fn WarningMessage(children: Children) -> impl IntoView {
    view! {
        <div class="notification is-warning">
            <div class="columns is-mobile is-vcentered">
                <div class="column is-narrow">
                    <i class="fa-solid fa-triangle-exclamation fa-2xl"></i>
                </div>
                <div class="column">
                    <p>{children()}</p>
                </div>
            </div>
        </div>
    }
}
