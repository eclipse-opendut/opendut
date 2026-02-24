#[cfg(feature = "client-auth")]
pub mod client_auth;

#[cfg(feature = "config")]
pub mod config;

#[cfg(feature = "crypto")]
pub mod crypto;

#[cfg(feature = "future")]
pub mod future;

#[cfg(all(feature = "pem", not(target_arch = "wasm32")))]
pub mod pem;

#[cfg(feature = "project")]
pub mod project;

#[cfg(feature = "proto")]
pub mod proto;

#[cfg(all(feature = "reqwest", not(target_arch = "wasm32")))]
pub mod reqwest_client;


#[cfg(feature = "serde")]
pub mod serde;

#[cfg(all(feature = "settings", not(target_arch = "wasm32")))]
pub mod settings;

#[cfg(feature = "testing")]
pub mod testing;

pub mod error;


pub fn expect_env_var(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("Environment variable {} is not set", key))
}
