mod licenses;
pub mod peers;
pub mod clusters;
#[cfg(feature = "viper")]
pub mod viper;

pub use licenses::{get_licenses, ApiError, ComponentLicenses};

use std::fmt::Display;

use opendut_carl_api::carl::ClientError;
use opendut_lea_components::ErrorText;
use tracing::error;

use crate::util::client_error_description;

fn client_error_text<E: Display>(
    error: &ClientError<E>,
    headline: &str,
    fallback: &str,
) -> ErrorText {
    error!("{headline}: {error}");

    ErrorText {
        headline: headline.to_owned(),
        description: client_error_description(error, fallback),
    }
}
