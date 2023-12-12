use crate::util::Ior;

pub mod readonly_input;
pub mod user_input;

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
