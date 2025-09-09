//! This crate serves as a component library for opendut-lea.
//!
//! All components which are generic enough so that
//! they could be used in another web-UI should
//! be extracted into this library.
//!
//! The goal is a clearer seperation to help with
//! structuring the code, but in particular also to
//! reduce compile times. By moving code into a separate
//! crate, we benefit from incremental compilation.


pub use breadcrumbs::{Breadcrumb, Breadcrumbs};
pub use buttons::button::SimpleButton;
pub use buttons::confirmation_button::ConfirmationButton;
pub use buttons::doorhanger_button::DoorhangerButton;
pub use buttons::icon_button::IconButton;
pub use buttons::collapse_button::CollapseButton;
pub use inputs::{UserInputError, UserInputValue};
pub use inputs::readonly_input::ReadOnlyInput;
pub use inputs::user_input::UserInput;
pub use inputs::user_textarea::UserTextarea;
pub use inputs::vector_user_input::VectorUserInput;
pub use loading_spinner::LoadingSpinner;
pub use page::BasePageContainer;
pub use toast::{use_toaster, Toast, ToastContent, ToastKind, Toaster};
pub use warning_message::WarningMessage;
pub use util::ior::Ior;
pub use util::net::UserNetworkInterfaceConfiguration;
pub use util::signal::{ButtonStateSignalProvider, Toggled};

pub mod health;
pub mod tooltip;

mod buttons;
mod breadcrumbs;
mod doorhanger;
mod inputs;
mod loading_spinner;
mod page;
mod toast;
mod warning_message;
mod util;

pub const NON_BREAKING_SPACE: &str = "\u{a0}";

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
