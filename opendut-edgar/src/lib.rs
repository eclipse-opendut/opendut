shadow_rs::shadow!(app_info);
shadow_formatted_version::from_shadow!(app_info);

mod cli;
pub use cli::cli;

mod common;
mod setup;
mod service;
pub(crate) use fs_err as fs;

///For integration tests
#[cfg(feature = "integration_testing")]
pub mod testing {
    pub use crate::common::settings;
    pub mod service {
        pub use crate::service::start;
        pub use crate::service::peer_messaging_client;
        pub use crate::service::peer_configuration;
    }
    pub mod carl {
        pub use crate::common::carl::connect;
    }
}
