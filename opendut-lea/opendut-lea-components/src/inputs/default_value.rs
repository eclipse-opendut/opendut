use leptos::prelude::*;
use crate::{Toggle, ToggleSignal};

#[component]
pub fn DefaultValue(
    #[prop(into)] default_value: Option<String>,
    use_default_value: RwSignal<bool>,
    children: Children,
) -> impl IntoView {

    view! {
        { children() }
        {
            default_value.map(|default_value| {
                view! {
                    <div class="is-flex is-justify-content-start">
                        <Toggle
                            left_text=format!("Use default (\"{}\"):", default_value)
                            is_active=use_default_value
                            on_action=move || {
                                use_default_value.toggle();
                            }
                        />
                    </div>
                }
            })
        }
    }
}
