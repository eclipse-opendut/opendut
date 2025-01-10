use crate::routing;
use leptos::prelude::*;
use leptos_router::hooks::use_location;
use crate::routing::WellKnownRoutes;

pub enum LeaError {
    ConfigurationError,
}

pub struct ErrorDetails {
    title: String,
    text: String,
}

impl ErrorDetails {
    pub fn new(title: impl Into<String>, text: impl Into<String>) -> ErrorDetails {
        ErrorDetails {
            title: title.into(),
            text: text.into(),
        }
    }
}

impl LeaError {
    pub fn details(&self) -> ErrorDetails {
        match self {
            LeaError::ConfigurationError => ErrorDetails::new("Configuration error", "Failed to load or parse configuration."),
        }
    }
    
    pub fn navigate_to(&self) {
        let details = self.details();
        
        routing::navigate_to(WellKnownRoutes::ErrorPage {
            title: details.title,
            text: details.text,
            details: None,
        })
    }
}

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
