use chrono::Local;
use leptos::prelude::*;
use crate::components::{Toast, ToastContent, Toaster, ToastKind};
use crate::components::toast::duration_as_ticks;

pub struct ToastBuilder {
    kind: Option<ToastKind>,
    content: Option<ToastContent>,
}

#[allow(dead_code)]
impl ToastBuilder {

    pub fn new() -> Self {
        Self {
            kind: Default::default(),
            content: Default::default(),
        }
    }

    pub fn kind(mut self, kind: ToastKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn info(mut self) -> Self {
        self.kind = Some(ToastKind::Info);
        self
    }

    pub fn success(mut self) -> Self {
        self.kind = Some(ToastKind::Success);
        self
    }

    pub fn warning(mut self) -> Self {
        self.kind = Some(ToastKind::Warning);
        self
    }

    pub fn error(mut self) -> Self {
        self.kind = Some(ToastKind::Error);
        self
    }

    pub fn content(mut self, content: ToastContent) -> Self {
        self.content = Some(content);
        self
    }

    pub fn simple(mut self, text: impl Into<String>) -> Self {
        self.content = Some(ToastContent::Simple { text: text.into() });
        self
    }
}

impl From<ToastBuilder> for Toast {
    fn from(value: ToastBuilder) -> Self {
        let ticks = duration_as_ticks(&Toast::DEFAULT_LIFETIME, Toaster::UPDATE_INTERVAL_IN_MILLIS);
        Toast {
            kind: value.kind.unwrap_or_else(|| ToastKind::Info),
            content: value.content.unwrap_or_else(|| ToastContent::Simple { text: String::new() }),
            timestamp: Local::now(),
            max_ticks: ticks,
            remaining_ticks: RwSignal::new(ticks),
            keep: RwSignal::new(false),
        }
    }
}
