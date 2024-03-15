use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "client")] {
        pub mod manager;
        pub mod service;
    }
}
#[cfg(feature = "oidc_client")]
pub mod auth_config;
