pub mod project;
pub mod future;
#[cfg(feature = "testing")]
pub mod testing;

#[cfg(all(feature = "settings", not(target_arch = "wasm32")))]
pub mod pem;
#[cfg(all(feature = "reqwest", not(target_arch = "wasm32")))]
pub mod reqwest_client;

pub fn expect_env_var(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("Environment variable {} is not set", key))
}