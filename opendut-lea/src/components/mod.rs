pub use breadcrumbs::{Breadcrumb, Breadcrumbs};
pub use buttons::button::SimpleButton;
pub use buttons::confirmation_button::ConfirmationButton;
pub use buttons::icon_button::IconButton;
pub use inputs::{UserInputError, UserInputValidator, UserInputValue};
pub use inputs::readonly_input::ReadOnlyInput;
pub use inputs::user_input::UserInput;
pub use page::BasePageContainer;
pub use util::ButtonStateSignalProvider;
pub use util::use_active_tab;

pub mod health;
pub mod tooltip;

mod doorhanger;
mod page;
mod util;
mod inputs;
mod buttons;
mod breadcrumbs;

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum FontAwesomeIcon {
    ArrowsRotate,
    Bars,
    Check,
    CircleNotch,
    EllipsisVertical,
    Plus,
    Save,
    TrashCan,
    User,
    XMark,
}

impl FontAwesomeIcon {

    pub fn as_class(&self) -> &'static str {
        match self {
            FontAwesomeIcon::ArrowsRotate => "fa-solid fa-arrows-rotate",
            FontAwesomeIcon::Bars => "fa-solid fa-bars",
            FontAwesomeIcon::Check => "fa-solid fa-check",
            FontAwesomeIcon::CircleNotch => "fa-solid fa-circle-notch",
            FontAwesomeIcon::EllipsisVertical => "fa-solid fa-ellipsis-vertical",
            FontAwesomeIcon::Plus => "fa-solid fa-plus",
            FontAwesomeIcon::Save => "fa-solid fa-save",
            FontAwesomeIcon::TrashCan => "fa-solid fa-trash-can",
            FontAwesomeIcon::User => "fa-solid fa-user-large",
            FontAwesomeIcon::XMark => "fa-solid fa-xmark",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
pub enum ButtonState {
    Enabled,
    Loading,
    Disabled,
    Hidden,
}

impl ButtonState {
    #[allow(non_upper_case_globals)]
    pub const Default: ButtonState = ButtonState::Enabled;
}

impl Default for ButtonState {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum ButtonColor {
    Danger,
    Info,
    Light,
    Success,
    White,
}

impl ButtonColor {

    pub fn as_class(&self) -> &'static str {
        match self {
            ButtonColor::Danger => "button is-danger",
            ButtonColor::Info => "button is-info",
            ButtonColor::Light => "button is-light",
            ButtonColor::Success => "button is-success",
            ButtonColor::White => "button is-white",
        }
    }
}
