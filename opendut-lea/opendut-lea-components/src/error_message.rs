use leptos::prelude::*;
use crate::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton};

#[derive(Clone, Debug)]
pub struct ErrorText {
    pub headline: String,
    pub description: String,
}

#[derive(Default)]
pub enum ErrorMessageType {
    #[default]
    Error,
    Warning,
}

#[component]
pub fn ErrorMessage(
    text: ErrorText,
    #[prop(optional)] error_type: ErrorMessageType
) -> impl IntoView {

    let color = match error_type {
        ErrorMessageType::Error => "is-danger",
        ErrorMessageType::Warning => "is-warning",
    };

    let icon = match error_type {
        ErrorMessageType::Error => "fa-solid fa-circle-xmark fa-lg",
        ErrorMessageType::Warning => "fa-solid fa-triangle-exclamation fa-lg",
    };

    view! {
        <article class=format!("message {color}")>
            <div class="message-body">
                <div class="columns is-mobile is-vcentered">
                    <div class="column is-narrow">
                        <i class=icon></i>
                    </div>
                    <div class="column">
                        <p class="has-text-weight-semibold">
                            {text.headline}
                        </p>
                        <p>{ text.description }</p>
                    </div>
                    <div class="column is-narrow">
                        <IconButton
                            icon=FontAwesomeIcon::Retry
                            color=ButtonColor::White
                            size=ButtonSize::Normal
                            state=ButtonState::Enabled
                            label="Reload Page"
                            on_action=move || {
                                let _ = window().location().reload(); // TODO: Maybe just refetch the data instead of reloading the whole page (-> causes rerendering)
                            }
                        />
                    </div>
                </div>
            </div>
        </article>
    }
}
