use crate::routing;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn ErrorPage() -> impl IntoView {
    let location = use_location();

    let title = move || {
        location.query.with(|query| {
            query
                .get("title")
                .unwrap_or(String::from("An error occurred"))
        })
    };

    let text = move || {
        location.query.with(|query| {
            query
                .get("text")
                .unwrap_or(String::from("We are sorry, but an unknown error occurred!"))
        })
    };

    let _details = move || location.query.with(|query| query.get("details"));

    view! {
        <div class="columns is-centered pt-2">
            <div class="column is-half">
                <div class="notification is-danger">
                    <p class="title is-4">{ title }</p>
                    <hr></hr>
                    <p class="subtitle is-5">{ text }</p>
                </div>
                <div class="is-flex is-justify-content-center">
                    <a class="button" href=routing::path::dashboard>"Return to Dashboard"</a>
                </div>
            </div>
        </div>
    }
}
