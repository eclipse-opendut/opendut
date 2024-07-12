#![cfg(test)]

mod register;
mod testing;
mod peer_configuration;


/// Selects which crypto provider to use.
/// Only needed while we have both `aws_lc_rs` and `ring` as transitive dependencies in the project.
/// https://github.com/rustls/rustls/issues/1938
pub fn install_crypto_provider() {
    let _ignore_errors = rustls::crypto::aws_lc_rs::default_provider().install_default(); //an error occurs when it has been installed previously, e.g. in a different unit test
}
