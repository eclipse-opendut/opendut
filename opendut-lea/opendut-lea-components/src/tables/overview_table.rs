use leptos::prelude::*;
use web_sys::MouseEvent;
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
    children: Children
) -> impl IntoView
where F: Fn(MouseEvent) + 'static {

    let on_row_click = move |event: MouseEvent| {
        if block_row_click.get_untracked() {
            block_row_click.set(false);
            return;
        }
        navigation_on_click(event);
    };

    view! {
        <tr class="is-clickable" on:click=on_row_click>
            { children() }
        </tr>
    }
}

#[component]
pub fn OverviewTableCell(children: Children) -> impl IntoView {
    view! {
        <td class="is-vcentered"> { children() } </td>
    }
}
