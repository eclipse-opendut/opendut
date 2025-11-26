use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use crate::routing::{navigate_to, WellKnownRoutes};

pub fn use_active_tab<
    T: for<'a> TryFrom<&'a str, Error=impl ToString>
    + Default + Send + Clone + Sync + PartialEq + 'static
>(
    tabs_in_page: Signal<Vec<T>>
) -> RwSignal<T> {

    let params = use_params_map();

    let active = RwSignal::new(T::default());

    Effect::new(move || {
        let tabs_in_page = tabs_in_page.get();

        params.with(|param| {
            let tab = param.get("tab")
                .ok_or(String::from("No tab identifier given in URL!"))
                .and_then(|value| T::try_from(value.as_str()).map_err(|cause| cause.to_string()));

            match tab {
                Ok(tab) if tabs_in_page.contains(&tab) => {
                    active.set(tab);
                }
                _ => {
                    let use_navigate = use_navigate();
                    navigate_to(WellKnownRoutes::ErrorPage {
                        title: String::from("Oops"),
                        text: String::from("You entered an invalid URL!"),
                        details: None, // Todo: Add details
                    }, use_navigate);
                    active.set(T::default());
                }
            }
        })
    });

    active
}
