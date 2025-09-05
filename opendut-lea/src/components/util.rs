use leptos::prelude::{Signal, With};
use leptos_router::hooks::{use_navigate, use_params_map};
use crate::routing::{navigate_to, WellKnownRoutes};

pub fn use_active_tab<
    T: for<'a> TryFrom<&'a str, Error=impl ToString>
       + Default + Send + Sync + 'static
>() -> Signal<T> {
    let params = use_params_map();

    Signal::derive(move || params.with(|params| {
        let tab = params.get("tab")
            .ok_or(String::from("No tab identifier given in URL!"))
            .and_then(|value| T::try_from(value.as_str()).map_err(|cause| cause.to_string()));
        match tab {
            Err(details) => {
                let use_navigate = use_navigate();

                navigate_to(WellKnownRoutes::ErrorPage {
                    title: String::from("Oops"),
                    text: String::from("You entered an invalid URL!"),
                    details: Some(details),
                }, use_navigate);
                T::default()
            }
            Ok(tab) => {
                tab
            }
        }
    }))
}
