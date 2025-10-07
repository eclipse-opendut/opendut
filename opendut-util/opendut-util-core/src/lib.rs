pub mod project;
pub mod future;
#[cfg(feature = "testing")]
pub mod testing;

pub fn expect_env_var(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("Environment variable {} is not set", key))
}