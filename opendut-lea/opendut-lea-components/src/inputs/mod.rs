use crate::util::ior::Ior;

pub mod readonly_input;
pub mod user_input;
pub mod user_textarea;
pub mod vector_user_input;
pub mod user_select;
pub mod default_value;

pub type UserInputError = String;
pub type UserInputValue = Ior<UserInputError, String>;

pub trait UserInputValidator {
    fn validate(&self, input: String) -> UserInputValue;
}

impl <A> UserInputValidator for A
    where A: Fn(String) -> UserInputValue + Clone {
    fn validate(&self, input: String) -> UserInputValue {
        (self)(input)
    }
}

pub enum InputType {
    Text,
    Number,
}

impl InputType {
    pub fn as_html_type(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Number => "number",
        }
    }
}
