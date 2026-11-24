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
    #[prop(into)] on_retry: Callback<()>,
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
                        <p style="white-space: pre-line;">{ text.description }</p>
                    </div>
                    <div class="column is-narrow">
                        <IconButton
                            icon=FontAwesomeIcon::Retry
                            color=ButtonColor::White
                            size=ButtonSize::Normal
                            state=ButtonState::Enabled
                            label="Retry"
                            on_action=move || on_retry.run(())
                        />
                    </div>
                </div>
            </div>
        </article>
    }
}

#[component]
pub fn ErrorMessageTableRow(
    text: ErrorText,
    #[prop(into)] on_retry: Callback<()>,
) -> impl IntoView {
    view! {
        <tr>
            <td colspan="999">
                <ErrorMessage
                    text
                    on_retry
                />
            </td>
        </tr>
    }
}
