use leptos::prelude::*;
use crate::{UserInputValue, NON_BREAKING_SPACE};

#[derive(Clone, Debug)]
pub struct SelectionTableRow {
    pub id: String,
    pub cells: Vec<String>,
}

#[component]
pub fn SelectionTable(
    #[prop(into)] header: Signal<Vec<String>>,
    #[prop(into)] rows: Signal<Vec<SelectionTableRow>>,
    getter: Signal<UserInputValue>,
    setter: SignalSetter<UserInputValue>,
) -> impl IntoView {

    let help_text = move || {
        getter.with(|selection| match selection {
            UserInputValue::Left(error) => error.to_owned(),
            UserInputValue::Right(_) => String::from(NON_BREAKING_SPACE),
            UserInputValue::Both(error, _) => error.to_owned(),
        })
    };

    let is_selected = move |id: String| {
        Signal::derive(move || {
            let getter = getter.get();
            match getter {
                UserInputValue::Right(selected) => id == selected,
                UserInputValue::Left(_) | UserInputValue::Both(_, _) => false,
            }
        })
    };

    view! {
        <p class="help has-text-danger"> { help_text } </p>
        <div class="table-container mt-2">
            <table class="table is-hoverable is-fullwidth">
                <thead>
                    <tr>
                        <For
                            each=move || header.get()
                            key=|header| header.to_owned()
                            children=move |column_name| {
                                view! {
                                    <th>{ column_name }</th>
                                }
                            }
                        />
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || rows.get()
                        key=|row| Clone::clone(&row.id)
                        children=move |row| {

                            let SelectionTableRow { id: row_id, cells } = row;
                            let is_selected = is_selected(Clone::clone(&row_id));

                            view! {
                                <tr
                                    class:has-background-link-light=move || is_selected.get()
                                    style="cursor: pointer;"
                                    on:click=move |_| {
                                        let row_id = Clone::clone(&row_id);
                                        setter.set(UserInputValue::Right(row_id));
                                    }
                                >
                                    <td class="is-narrow">
                                        <div class="control">
                                            <label class="radio">
                                                <input
                                                    type="radio"
                                                    name="selected-cluster"
                                                    prop:checked=is_selected
                                                />
                                            </label>
                                        </div>
                                    </td>
                                    <For
                                        each=move || Clone::clone(&cells)
                                        key=|cell| cell.to_owned()
                                        children=move |cell| {
                                            view! {
                                                <td> { cell } </td>
                                            }
                                        }
                                    />
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        </div>
    }
}
