use leptos::prelude::*;
use crate::{ButtonColor, ButtonSize};

#[component]
pub fn Tag(
    #[prop(into)] text: String,
    #[prop(default=ButtonColor::Light)] color: ButtonColor,
    #[prop(default=ButtonSize::Normal)] size: ButtonSize,
    #[prop(optional)] on_delete: Option<Callback<()>>,
) -> impl IntoView {

    let tag_class = format!(
        "tag {} {}",
        color.as_class(),
        size.as_class(),
    );

    match on_delete {
        Some(on_delete) => {
            view! {
                <div class="control">
                    <div class="tags has-addons">
                        <span class=tag_class> { text } </span>
                        <a
                            class="tag is-delete"
                            on:click=move |_| on_delete.run(())
                        />
                    </div>
                </div>
            }.into_any()
        }
        None => {
            view! {
                <span class=tag_class>
                    { text }
                </span>
            }.into_any()
        }
    }
}
