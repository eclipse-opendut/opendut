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
pub use util::signal::{ButtonStateSignalProvider, Toggled, ToggleSignal};
pub use doorhanger::{Doorhanger, DoorhangerAlignment};
pub use icon_text::IconText;
pub use toggle::Toggle;

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
mod icon_text;
mod toggle;

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
    Cluster,
    Dashboard,
    Downloads,
    EllipsisVertical,
    Email,
    OpenPage,
    Peers,
    Plus,
    Save,
    SignOut,
    TrashCan,
    User,
    UserOutlined,
    Users,
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
            FontAwesomeIcon::Cluster => "fa-solid fa-circle-nodes",
            FontAwesomeIcon::Dashboard => "fa-solid fa-gauge-high",
            FontAwesomeIcon::Downloads => "fa-solid fa-download",
            FontAwesomeIcon::EllipsisVertical => "fa-solid fa-ellipsis-vertical",
            FontAwesomeIcon::Email => "fa-regular fa-envelope",
            FontAwesomeIcon::OpenPage => "fa-solid fa-arrow-up-right-from-square",
            FontAwesomeIcon::Peers => "fa-solid fa-microchip",
            FontAwesomeIcon::Plus => "fa-solid fa-plus",
            FontAwesomeIcon::Save => "fa-solid fa-save",
            FontAwesomeIcon::SignOut => "fas fa-sign-out-alt",
            FontAwesomeIcon::TrashCan => "fa-solid fa-trash-can",
            FontAwesomeIcon::User => "fa-solid fa-user-large",
            FontAwesomeIcon::UserOutlined => "fa-regular fa-user",
            FontAwesomeIcon::Users =>"fa-solid fa-users",
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

impl ButtonState {
    pub fn as_class(&self) -> &'static str {
        match self {
            ButtonState::Loading => "is-loading",
            ButtonState::Hidden => "is-hidden",
            _ => ""
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum ButtonColor {
    None,
    Danger,
    Info,
    Light,
    Primary,
    Success,
    TextDanger,
    Warning,
    White,
}

impl ButtonColor {

    pub fn as_class(&self) -> &'static str {
        match self {
            ButtonColor::None => "is-text",
            ButtonColor::Danger => "is-danger",
            ButtonColor::Info => "is-info",
            ButtonColor::Light => "is-light",
            ButtonColor::Primary => "is-primary",
            ButtonColor::Success => "is-success",
            ButtonColor::TextDanger => "is-white has-text-danger",
            ButtonColor::Warning => "is-warning",
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

#[derive(Clone, Copy)]
pub struct Hsl(pub i32, pub i32, pub i32);

pub enum ProfilePictureColors {
    DarkRed,
    Red,
    Orange,
    Yellow,
    LightGreen,
    DarkGreen,
    LightBlue,
    Blue,
    DarkBlue,
    Purple,
    Pink,
    Grey,
    Brown
}

impl ProfilePictureColors {
    pub fn get_hsl(&self) -> Hsl {
        match self {
            Self::DarkRed => Hsl(355, 76, 36),
            Self::Red => Hsl(356, 83, 41),
            Self::Orange => Hsl(31, 100, 48),
            Self::Yellow => Hsl(41, 100, 53),
            Self::LightGreen => Hsl(78, 51, 38),
            Self::DarkGreen => Hsl(146, 43, 30),
            Self::LightBlue => Hsl(173, 65, 40),
            Self::Blue => Hsl(217, 100, 56),
            Self::DarkBlue => Hsl(220, 54, 25),
            Self::Purple => Hsl(294, 42, 42),
            Self::Pink => Hsl(334, 100, 50),
            Self::Grey => Hsl(0, 0, 35),
            Self::Brown => Hsl(20, 18, 44),
        }
    }

    pub fn get_vec() -> Vec<Self> {
        vec![
            Self::DarkRed,
            Self::Red,
            Self::Orange,
            Self::Yellow,
            Self::LightGreen,
            Self::DarkGreen,
            Self::LightBlue,
            Self::Blue,
            Self::DarkBlue,
            Self::Purple,
            Self::Pink,
            Self::Grey,
            Self::Brown,
        ]
    }
}
