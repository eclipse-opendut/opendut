use leptos::prelude::*;
use opendut_lea_components::{ErrorIndicator, ErrorText};

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
                                view! {
                                    <ErrorIndicator text=error />
                                }.into_any()
                            }
                        }}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}
