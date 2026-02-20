use std::hash::Hash;
use leptos::prelude::*;
use crate::{Ior, NON_BREAKING_SPACE};

#[derive(Clone, Debug)]
pub struct SelectionTableRow<Id> {
    pub id: Id,
    pub cells: Vec<String>,
}

#[component]
pub fn SelectionTable<Id>(
    #[prop(into)] header: Signal<Vec<String>>,
    #[prop(into)] rows: Signal<Vec<SelectionTableRow<Id>>>,
    getter: Signal<Ior<String, Id>>,
    setter: SignalSetter<Ior<String, Id>>,
    #[prop(into, default=Signal::from(false))] is_disabled: Signal<bool>,
) -> impl IntoView
where
    Id: Clone + Eq + ToString + Send + Sync + Hash + 'static,
{

    let help_text = move || {
        getter.with(|selection| match selection {
            Ior::Left(error) => error.to_owned(),
            Ior::Right(_) => String::from(NON_BREAKING_SPACE),
            Ior::Both(error, _) => error.to_owned(),
        })
    };

    view! {
        <p class="help has-text-danger"> { help_text } </p>
        <div class="dut-table-container">
            <table class="dut-table">
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
                            let is_selected = {
                                let row_id = Clone::clone(&row_id);
                                Signal::derive(move || {
                                    let getter = getter.get();
                                    match getter {
                                        Ior::Right(selected) => row_id == selected,
                                        Ior::Left(_) | Ior::Both(_, _) => false,
                                    }
                                })
                            };

                            view! {
                                <tr
                                    class="dut-row"
                                    class=("selected", move || is_selected.get())
                                    class=("disabled", move || is_disabled.get())
                                    on:click=move |_| {
                                        if is_disabled.get() { return }
                                        let row_id = Clone::clone(&row_id);
                                        setter.set(Ior::Right(row_id));
                                    }
                                >
                                    <td class="is-narrow has-text-centered">
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
