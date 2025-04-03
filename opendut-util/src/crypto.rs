use tracing::trace;

/// Selects which crypto provider to use.
/// Only needed while we have both `aws_lc_rs` and `ring` as transitive dependencies in the project.
/// <https://github.com/rustls/rustls/issues/1938>
pub fn install_default_provider() {
    let result = rustls::crypto::ring::default_provider().install_default();

    if result.is_ok() { //ignore errors, as these occur when it was previously already installed, e.g. in a different unit test
        trace!("Installed ring as default crypto provider.");
    }
}
