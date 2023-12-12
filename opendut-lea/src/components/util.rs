use leptos::{MaybeSignal, ReadSignal, Signal, SignalGet, SignalWith};
use leptos_router::use_params_map;

use crate::components::ButtonState;
use crate::routing::{navigate_to, WellKnownRoutes};

pub trait ButtonStateSignalProvider {
    fn derive_loading(&self) -> MaybeSignal<ButtonState>;
}

impl ButtonStateSignalProvider for ReadSignal<bool> {
    fn derive_loading(&self) -> MaybeSignal<ButtonState> {
        derive_loading(self)
    }
}

impl ButtonStateSignalProvider for Signal<bool> {
    fn derive_loading(&self) -> MaybeSignal<ButtonState> {
        derive_loading(self)
    }
}

fn derive_loading(signal: &(impl SignalGet<Value=bool> + Clone + 'static)) -> MaybeSignal<ButtonState> {
    let signal = Clone::clone(signal);
    MaybeSignal::derive(move || {
        if signal.get() {
            ButtonState::Loading
        }
        else {
            ButtonState::Default
        }
    })
}

pub fn use_active_tab<T: for<'a> TryFrom<&'a str, Error=impl ToString> + Default>() -> MaybeSignal<T> {
    let params = use_params_map();
    MaybeSignal::derive(move || params.with(|params| {
        let tab = params.get("tab")
            .ok_or(String::from("No tab identifier given in URL!"))
            .and_then(|value| T::try_from(value.as_str()).map_err(|cause| cause.to_string()));
        match tab {
            Err(details) => {
                navigate_to(WellKnownRoutes::ErrorPage {
                    title: String::from("Oops"),
                    text: String::from("You entered an invalid URL!"),
                    details: Some(details),
                });
                T::default()
            }
            Ok(tab) => {
                tab
            }
        }
    }))
}
