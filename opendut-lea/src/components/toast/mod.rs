use std::ops::Not;
use std::rc::Rc;
use std::time::Duration;

use chrono::{DateTime, Local};
use leptos::{create_effect, create_rw_signal, document, IntoView, RwSignal, SignalGet, SignalGetUntracked, SignalSet, SignalUpdate, SignalWith, use_context, view, web_sys};
use leptos::leptos_dom::Mountable;
use leptos::web_sys::Node;
use leptos_use::use_interval_fn;
use leptos_use::utils::Pausable;
use slotmap::{DefaultKey, SlotMap};
use tracing::{debug, info};

use crate::components::toast::builder::ToastBuilder;
use crate::components::toast::container::Container;
use crate::util::Tick;

mod notification;
mod container;
mod builder;

type ToastKey = DefaultKey;
type ToastMap = SlotMap<ToastKey, RwSignal<Toast>>;

#[derive(Clone, Debug)]
pub struct Toast {
    kind: ToastKind,
    content: ToastContent,
    timestamp: DateTime<Local>,
    max_ticks: Tick,
    remaining_ticks: RwSignal<Tick>,
    keep: RwSignal<bool>,
}

impl Toast {

    const DEFAULT_LIFETIME: Duration = Duration::from_secs(3);

    pub fn builder() -> ToastBuilder {
        ToastBuilder::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error
}

#[derive(Clone, Debug, PartialEq)]
pub enum ToastContent {
    Simple {
        text: String
    }
}

pub fn use_toaster() -> Rc<Toaster> {
    use_context::<Rc<Toaster>>()
        .expect("The Toaster should be provided in the context.")
}

pub struct Toaster {
    toasts: RwSignal<ToastMap>,
}

impl Toaster {
    const UPDATE_INTERVAL_IN_MILLIS: u64 = 250;

    pub fn new() -> Self {

        debug!("Creating toaster.");

        let parent: web_sys::HtmlElement = document().body()
            .expect("There should be a html body");

        let toasts: RwSignal<ToastMap> = create_rw_signal(Default::default());

        let Pausable { pause: pause_toast_janitor, resume: resume_toast_janitor, is_active: is_toast_janitor_active } = use_interval_fn(move || {
            toasts.update(|toasts: &mut ToastMap| {
                let mut toasts_to_remove = Vec::<ToastKey>::new();
                toasts.iter().for_each(|(key, toast)| {
                    toast.with(|toast| {
                        let remaining_ticks = toast.remaining_ticks.get_untracked();
                        let keep = toast.keep.get_untracked();
                        if remaining_ticks < 1 && keep.not() {
                            toasts_to_remove.push(key);
                        }
                        else if remaining_ticks > 0 && keep.not() {
                            toast.remaining_ticks.set(remaining_ticks.saturating_sub(1.into()));
                        }
                    });
                });
                toasts_to_remove.iter()
                    .for_each(|key| {
                        toasts.remove(*key);
                    });
            });
        }, Self::UPDATE_INTERVAL_IN_MILLIS);

        create_effect(move |_| {
            toasts.with(|toasts| {
                let is_active = is_toast_janitor_active.get();
                if toasts.is_empty() {
                    pause_toast_janitor();
                    debug!("Toast-Janitor paused.");
                }
                else if is_active.not() {
                    resume_toast_janitor();
                    debug!("Toast-Janitor resumed.");
                }
            });
        });

        let container: Node = {
            view! { <Container toasts /> }
        }.into_view().get_mountable_node();

        let _ = parent.append_child(&container)
            .expect("Toaster should be appendable to the body");

        info!("Toaster created.");

        Self {
            toasts,
        }
    }

    pub fn toast(&self, toast: impl Into<Toast>) {
        let toast = toast.into();
        debug!("{toast:?}");
        self.toasts.update(|toasts| {
            toasts.insert(create_rw_signal(toast));
        });
    }
}

fn duration_as_ticks(duration: &Duration, interval_ms: u64) -> Tick {
    Tick::from(duration.as_millis() / interval_ms as u128)
}

impl From<String> for Toast {
    fn from(value: String) -> Self {
        let ticks = duration_as_ticks(&Toast::DEFAULT_LIFETIME, Toaster::UPDATE_INTERVAL_IN_MILLIS);
        Toast {
            kind: ToastKind::Info,
            content: ToastContent::Simple { text: value },
            timestamp: Local::now(),
            max_ticks: ticks,
            remaining_ticks: create_rw_signal(ticks),
            keep: create_rw_signal(false),
        }
    }
}

impl From<&str> for Toast {
    fn from(value: &str) -> Self {
        From::from(String::from(value))
    }
}
