use leptos::prelude::*;
use crate::LoadingSpinner;

#[derive(Clone)]
pub struct TableHeading {
    pub title: String,
    pub is_narrow: bool,
}

impl TableHeading {
    pub fn new(title: String) -> Self {
        Self {
            title,
            is_narrow: false,
        }
    }
    pub fn set_narrow(self) -> Self {
        Self {
            title: self.title,
            is_narrow: true,
        }
    }
}

#[component]
pub fn OverviewTable(
    headings: Vec<TableHeading>,
    children: Children,
) -> impl IntoView {

    view! {
        <table class="table is-hoverable is-fullwidth">
            <thead>
                <tr>
                    <For
                        each=move || Clone::clone(&headings)
                        key=|heading| Clone::clone(&heading.title)
                        children=move |heading| {
                            let title = heading.title;
                            let is_narrow = heading.is_narrow;

                            view! {
                                <th class=(["is-narrow", "has-text-centered"], is_narrow)>
                                    { title }
                                </th>
                            }
                        }
                    />
                </tr>
            </thead>
            <tbody>
                <Suspense fallback=LoadingSpinner>
                    { children() }
                </Suspense>
            </tbody>
        </table>
    }
}

#[component]
pub fn OverviewTableRow<F>(
    block_row_click: RwSignal<bool>,
    navigation_on_click: F,
    children: ChildrenFragment
) -> impl IntoView
where F: Fn() + 'static {

    let children = children()
        .nodes
        .into_iter()
        .map(|child| view! { <td class="is-vcentered"> { child } </td> })
        .collect::<Vec<_>>();

    let on_row_click = move |_| {
        if block_row_click.get_untracked() {
            block_row_click.set(false);
            return;
        }
        navigation_on_click();
    };

    view! {
        <tr class="is-clickable" on:click=on_row_click>
            { children }
        </tr>
    }
}
