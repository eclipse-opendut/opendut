use leptos::prelude::*;

use crate::{Breadcrumbs, Breadcrumb};

#[component]
pub fn BasePageContainer<C>(
    #[prop(into)] title: Signal<String>,
    #[prop(into, optional)] subtitle: Signal<String>,
    #[prop(into)] breadcrumbs: Signal<Vec<Breadcrumb>>,
    controls: C,
    children: Children
) -> impl IntoView
where C: IntoView + 'static {

    view! {
        <div class="container is-fluid">
            <Breadcrumbs breadcrumbs />
            <div class="dut-base-page">
                <div class="dut-base-page-header columns is-vcentered mb-0">
                    <div class="column">
                        <p class="dut-base-page-title mb-0">{ title }" "</p>
                        {
                            if !subtitle.read().is_empty() {
                                Some(view! {
                                    <p class="dut-base-page-subtitle mt-0">{ subtitle }</p>
                                })
                            } else {
                                None
                            }
                        }
                    </div>
                    <div class="column is-narrow">
                        { controls }
                    </div>
                </div>
                <div class="dut-base-page-content">
                    { children() }
                </div>
            </div>
        </div>
    }
}
