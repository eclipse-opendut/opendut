use std::sync::Once;
use crate::project;

static INIT: Once = Once::new();

/// Loads the .env file once for all tests.
pub fn init_localenv_secrets() {
    let secrets_path = project::make_path_absolute(".ci/deploy/localenv/data/secrets/.env")
        .expect("Could not resolve secrets");
    INIT.call_once(|| {
        dotenvy::from_path(secrets_path).ok();
    });
}

