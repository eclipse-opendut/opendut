use leptos::prelude::*;
use crate::ButtonState;


pub trait ButtonStateSignalProvider {
    fn derive_loading(self) -> Signal<ButtonState>;
}

impl ButtonStateSignalProvider for ReadSignal<bool> {
    fn derive_loading(self) -> Signal<ButtonState> {
        let signal = Signal::from(self);
        derive_loading(signal)
    }
}

impl ButtonStateSignalProvider for Signal<bool> {
    fn derive_loading(self) -> Signal<ButtonState> {
        derive_loading(self)
    }
}

fn derive_loading(signal: Signal<bool>) -> Signal<ButtonState> {
    Signal::derive(move || {
        if signal.get() {
            ButtonState::Loading
        }
        else {
            ButtonState::Enabled
        }
    })
}


#[allow(unused)]
pub trait ToggleSignal {
    fn toggle(&self);
}

impl ToggleSignal for RwSignal<bool> {
    fn toggle(&self) {
        self.update(|value| *value = !*value)
    }
}

pub trait Toggled {
    fn derive_toggled<T>(self, on: T, off: T) -> Signal<T>
    where T: Clone + Send + Sync + 'static;
}

impl Toggled for ReadSignal<bool> {
    fn derive_toggled<T>(self, on: T, off: T) -> Signal<T>
    where
        T: Clone + Send + Sync + 'static
    {
        derive_toggled(self.into(), on, off)
    }
}

impl Toggled for Signal<bool> {
    fn derive_toggled<T>(self, on: T, off: T) -> Signal<T>
    where
        T: Clone + Send + Sync + 'static
    {
        derive_toggled(self, on, off)
    }
}

impl Toggled for RwSignal<bool> {
    fn derive_toggled<T>(self, on: T, off: T) -> Signal<T>
        where
            T: Clone + Send + Sync + 'static
    {
        let signal = Signal::from(self);
        derive_toggled(signal, on, off)
    }
}

fn derive_toggled<T>(signal: Signal<bool>, on: T, off: T) -> Signal<T>
where
    T: Clone + Send + Sync + 'static
{
    Signal::derive(move || {
        if signal.get() {
            Clone::clone(&on)
        }
        else {
            Clone::clone(&off)
        }
    })
}
