pub use auth::LeaAuthenticated;
pub use authenticated::{Initialized, AppGlobalsResource};
pub use breadcrumbs::{Breadcrumb, Breadcrumbs};
pub use buttons::button::SimpleButton;
pub use buttons::confirmation_button::ConfirmationButton;
pub use buttons::doorhanger_button::DoorhangerButton;
pub use buttons::icon_button::IconButton;
pub use generate_setup_string::{GenerateSetupStringForm, GenerateSetupStringKind};
pub use inputs::{UserInputError, UserInputValue};
pub use inputs::readonly_input::ReadOnlyInput;
pub use inputs::user_input::UserInput;
pub use inputs::user_textarea::UserTextarea;
pub use inputs::vector_user_input::VectorUserInput;
pub use loading_spinner::LoadingSpinner;
pub use page::BasePageContainer;
pub use toast::{Toast, ToastContent, Toaster, ToastKind, use_toaster};
pub use util::ButtonStateSignalProvider;
pub use util::Toggled;
pub use util::use_active_tab;
pub use warning_message::WarningMessage;

pub mod health;
pub mod tooltip;

pub mod auth;
mod authenticated;
mod buttons;
mod breadcrumbs;
mod doorhanger;
mod generate_setup_string;
mod inputs;
mod loading_spinner;
mod page;
mod toast;
mod util;
mod warning_message;

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum FontAwesomeIcon {
    ArrowsRotate,
    Bars,
    Check,
    ChevronDown,
    ChevronUp,
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
            FontAwesomeIcon::ChevronDown => "fa-solid fa-chevron-down",
            FontAwesomeIcon::ChevronUp => "fa-solid fa-chevron-up",
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

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[allow(dead_code)]
pub enum ButtonState {
    #[default]
    Enabled,
    Loading,
    Disabled,
    Hidden,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum ButtonColor {
    None,
    Danger,
    Info,
    Light,
    Success,
    White,
}

impl ButtonColor {

    pub fn as_class(&self) -> &'static str {
        match self {
            ButtonColor::None => "is-text",
            ButtonColor::Danger => "is-danger",
            ButtonColor::Info => "is-info",
            ButtonColor::Light => "is-light",
            ButtonColor::Success => "is-success",
            ButtonColor::White => "is-white",
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum ButtonSize {
    Small,
    Normal,
    Medium,
    Large,
}

impl ButtonSize {

    pub fn as_class(&self) -> &'static str {
        match self {
            ButtonSize::Small => "is-small",
            ButtonSize::Normal => "",
            ButtonSize::Medium => "is-medium",
            ButtonSize::Large => "is-large",
        }
    }
}
