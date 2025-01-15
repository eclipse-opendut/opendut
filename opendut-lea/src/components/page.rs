use leptos::prelude::*;

use crate::components::{Breadcrumbs, Breadcrumb};

#[component]
pub fn BasePageContainer<C>(
    #[prop(into)] title: Signal<String>,
    #[prop(into)] breadcrumbs: Signal<Vec<Breadcrumb>>,
    controls: C,
    children: Children
) -> impl IntoView
where C: IntoView + 'static {

    view! {

        <div class="container is-fluid">
            <Breadcrumbs breadcrumbs=breadcrumbs />
            <div class="dut-base-page">
                <div class="dut-base-page-header columns is-vcentered">
                    <div class="column">
                        <span class="dut-base-page-title is-4">{ title }</span>
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
