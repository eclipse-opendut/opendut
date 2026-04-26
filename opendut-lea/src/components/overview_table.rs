use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen_futures::wasm_bindgen::JsCast;
use web_sys::{Element, MouseEvent};
use opendut_lea_components::OverviewTableRow;
use crate::app::SelectionContext;

#[component]
pub fn ClickableOverviewTableRow(
    configurator_href: Signal<String>,
    children: Children,
) -> impl IntoView {

    let selection = use_context::<SelectionContext>()
        .expect("SelectionContext should be provided in the context.");

    let block_row_click = selection.has_selection;

    let use_navigate = use_navigate();
    let navigation_on_click = move |event: MouseEvent| {
        let Some(target) = event.target() else {
            return;
        };

        let Ok(element) = target.dyn_into::<Element>() else {
            return;
        };

        let Ok(Some(_)) = element.closest("a, button, input") else {
            use_navigate(&configurator_href.get(), Default::default());
            return;
        };
    };

    view! {
        <OverviewTableRow
            block_row_click
            navigation_on_click
            children
        />
    }
}
