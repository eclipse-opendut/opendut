pub use overview::UserOverview;
/// In case authentication is disabled the user identity is not known
pub const UNAUTHENTICATED_USER: &str = "unknown-user";
mod overview;
