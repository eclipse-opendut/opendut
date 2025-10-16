use leptos::prelude::*;

use crate::{Breadcrumbs, Breadcrumb};

#[component]
pub fn BasePageContainer<C>(
    #[prop(into)] title: Signal<String>,
    #[prop(into, optional)] subtitle: Option<Signal<String>>,
    #[prop(into)] breadcrumbs: Signal<Vec<Breadcrumb>>,
    controls: C,
    children: Children
) -> impl IntoView
where C: IntoView + 'static {

    Effect::new(move || {
        let title = title.get();
        let subtitle = subtitle.get().map(|subtitle| {
            if subtitle.is_empty() {
                String::new()
            } else {
                format!("- {subtitle}")
            }
        }).unwrap_or_default();

        let page_title = move || format!("{} {}", title, subtitle);
        leptos::leptos_dom::helpers::document().set_title(&page_title());
    });

    view! {
        <div class="container is-fluid">
            <Breadcrumbs breadcrumbs />
            <div class="dut-base-page">
                <div class="dut-base-page-header columns is-vcentered mb-0">
                    <div class="column">
                        <p class="dut-base-page-title mb-0">{ title }" "</p>
                        { move || {
                            subtitle.get().map(|subtitle| {
                                view! {
                                    <p class="dut-base-page-subtitle mt-0">{ subtitle }</p>
                                }
                            })
                        }}
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
