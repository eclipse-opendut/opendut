use leptos::prelude::*;

#[component]
pub fn LoadingSpinner() -> impl IntoView {
    view! {
        <div>
            <span class="icon">
                <i class="fa-spin fa-solid fa-circle-notch" />
            </span>
        </div>
    }
}
