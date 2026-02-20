use std::collections::HashSet;
use std::hash::Hash;
use leptos::prelude::*;
use crate::{ButtonColor, ButtonSize, ButtonState, CollapseButton, FontAwesomeIcon, IconButton, Ior, NON_BREAKING_SPACE};

#[derive(Clone, Debug)]
pub struct CollapsableInfo {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct MultipleSelectionTableRow<Id> {
    pub id: Id,
    pub cells: Vec<String>,
    pub details: Vec<CollapsableInfo>,
}

#[component]
pub fn MultipleSelectionTable<Id>(
    #[prop(into)] header: Signal<Vec<String>>,
    #[prop(into)] rows: Signal<Vec<MultipleSelectionTableRow<Id>>>,
    getter: Signal<Ior<String, HashSet<Id>>>,
    setter: SignalSetter<Ior<String, HashSet<Id>>>,
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
                        <th />
                        <For
                            each=move || header.get()
                            key=|header| header.to_owned()
                            children=move |column_name| {
                                view! {
                                    <th>{ column_name }</th>
                                }
                            }
                        />
                        <th />
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || rows.get()
                        key=|row| Clone::clone(&row.id)
                        children=move |row| {
                            let is_collapsed = RwSignal::new(true);

                            let MultipleSelectionTableRow { id: row_id, cells, details } = row;
                            let is_selected = {
                                let row_id = Clone::clone(&row_id);
                                Signal::derive(move || {
                                    let getter = getter.get();
                                    match getter {
                                        Ior::Right(selected) | Ior::Both(_, selected) => selected.contains(&row_id),
                                        Ior::Left(_) => false,
                                    }
                                })
                            };

                            view! {
                                <tr
                                    class="dut-row"
                                    class=("selected", move || is_selected.get())
                                    class=("disabled", move || is_disabled.get())
                                    on:click=move |_| {
                                        if is_disabled.get() { return; }
                                        let id = row_id.clone();

                                        let next = match getter.get() {
                                            Ior::Left(_) => {
                                                let mut hash_set = HashSet::new();
                                                hash_set.insert(id);
                                                Ior::Both(String::from("Select at least one more device."), hash_set)
                                            }
                                            Ior::Right(mut devices) | Ior::Both(_, mut devices) => {
                                                if !devices.insert(id.clone()) {
                                                    devices.remove(&id);
                                                }
                                                match devices.len() {
                                                    0 => Ior::Left(String::from("Select at least two devices.")),
                                                    1 => Ior::Both(String::from("Select at least one more device."), devices),
                                                    _ => Ior::Right(devices),
                                                }
                                            }
                                        };

                                        setter.set(next);
                                    }
                                >
                                    <td class="is-narrow">
                                        <CollapseButton collapsed=is_collapsed label="Show or hide device details" />
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
                                    <td class="is-narrow">
                                        <IconButton
                                            icon=FontAwesomeIcon::Check
                                            color=Signal::derive(move || if is_selected.get() {
                                                ButtonColor::Info
                                            } else {
                                                ButtonColor::Light
                                            })
                                            size=ButtonSize::Small
                                            state=ButtonState::Enabled
                                            label="Add device to cluster"
                                            skip_stop_propagation=true
                                            on_action=|| {}
                                        />
                                    </td>
                                </tr>
                                <Show when=move || !is_collapsed.get()>
                                    <tr>
                                        <CollapsableInfoView
                                            info=Clone::clone(&details)
                                        />
                                    </tr>
                                </Show>
                            }
                        }
                    />
                </tbody>
            </table>
        </div>
    }
}

#[component]
fn CollapsableInfoView(#[prop(into)] info: Vec<CollapsableInfo>) -> impl IntoView {
    view! {
        <td></td>
        <td colspan="3">
            <For
                each=move || Clone::clone(&info)
                key=|info| Clone::clone(&info.label)
                children=move |info| {
                    let CollapsableInfo { label, value } = info;
                    view! {
                        <div class="field">
                            <label class="label"> { label } </label>
                            <div class="control">
                                <p>{ value }</p>
                            </div>
                        </div>
                    }
                }
            />
        </td>
        <td></td>
    }
}
