use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use opendut_lea_components::OverviewTableRow;
use crate::app::SelectionContext;

#[component]
pub fn LeaOverviewTableRow(
    configurator_href: String,
    children: ChildrenFragment,
) -> impl IntoView {
    
    let selection = use_context::<SelectionContext>()
        .expect("SelectionContext should be provided in the context.");

    let block_row_click = selection.has_selection;

    let use_navigate = use_navigate();
    let navigation_on_click = move || {
        use_navigate(&configurator_href, Default::default());
    };

    view! {
        <OverviewTableRow
            block_row_click
            navigation_on_click
            children
        />
    }
}
