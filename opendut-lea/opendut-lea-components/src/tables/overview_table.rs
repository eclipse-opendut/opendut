use leptos::prelude::*;
use crate::LoadingSpinner;

#[derive(Clone)]
pub struct TableHeading {
    pub title: String,
    pub is_narrow: bool,
}

impl TableHeading {
    pub fn new(title: String, is_narrow: bool) -> Self {
        Self {
            title,
            is_narrow,
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
                                <th class=("is-narrow has-text-centered", is_narrow)>
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
