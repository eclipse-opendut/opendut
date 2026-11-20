use std::sync::Arc;
use leptos::prelude::*;
use opendut_lea_components::{ErrorText, FontAwesomeIcon};
use opendut_lea_components::tooltip::{Tooltip, TooltipContent};

#[component]
pub fn DashboardCardItem(
    #[prop(into)] title: String,
    value: Signal<Option<Result<usize, ErrorText>>>,
) -> impl IntoView {

    view! {
        <div class="level-item has-text-centered">
            <div>
                <p class="heading">{title.clone()}</p>
                <div class="title">
                    <Suspense fallback=move || view! { <span> "-" </span> }>
                        { move || match value.get() {
                            None => view! { <span> "-" </span> }.into_any(),
                            Some(Ok(value)) => view! { <span>{value}</span> }.into_any(),
                            Some(Err(error)) => {
                                let error_label = format!("Could not load {}", title);
                                let tooltip: TooltipContent = Arc::new(move || {
                                    view! {
                                        <span class="has-text-left has-text-weight-normal">
                                            <p> { error.headline.clone() } </p>
                                            <p> { error.description.clone() } </p>
                                        </span>
                                    }.into_any()
                                });

                                view! {
                                    <Tooltip text=tooltip>
                                        <span
                                            class="has-text-danger"
                                            role="img"
                                            aria-label=error_label
                                        >
                                            <i class=FontAwesomeIcon::TriangleExclamation.as_class() aria-hidden="true" />
                                        </span>
                                    </Tooltip>
                                }.into_any()
                            }
                        }}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}
