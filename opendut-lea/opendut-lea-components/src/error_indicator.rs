use std::sync::Arc;
use leptos::prelude::*;
use crate::{ErrorText, FontAwesomeIcon};
use crate::tooltip::{Tooltip, TooltipContent, TooltipDirection};

#[component]
pub fn ErrorIndicator(
    text: ErrorText,
    #[prop(into, optional)] direction: Signal<TooltipDirection>,
) -> impl IntoView {
    let label = if text.description.is_empty() {
        text.headline.clone()
    } else {
        format!("{}: {}", text.headline, text.description)
    };

    let tooltip: TooltipContent = Arc::new(move || {
        view! {
            <div class="has-text-left has-text-weight-normal" style="white-space: pre-line;">
                <p>{text.headline.clone()}</p>
                {(!text.description.is_empty()).then(|| view! {
                    <p>{text.description.clone()}</p>
                })}
            </div>
        }.into_any()
    });

    view! {
        <Tooltip text=tooltip direction>
            <span class="has-text-danger" role="img" aria-label=label>
                <i class=format!("fa-lg {}", FontAwesomeIcon::TriangleExclamation.as_class()) aria-hidden="true" />
            </span>
        </Tooltip>
    }
}
