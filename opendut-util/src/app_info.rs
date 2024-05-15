/// Generates an `app_info` module with information collected at build time.
///
/// Has to be called in the `main.rs` or `lib.rs` of your crate.
#[macro_export]
macro_rules! app_info {
    () => {
        shadow_rs::shadow!(__build_info);

        pub mod app_info {
            use super::__build_info;

            pub const CRATE_VERSION: &'static str = __build_info::PKG_VERSION;
            pub const BUILD_TARGET: &'static str = __build_info::BUILD_TARGET;
            pub const REVISION: &'static str = __build_info::COMMIT_HASH;
            pub const REVISION_DATE: &'static str = __build_info::COMMIT_DATE;
            pub const BUILD_DATE: &'static str = __build_info::BUILD_TIME;
            pub const BUILD_CLEAN: &'static str = if __build_info::GIT_CLEAN {
                "No uncommitted changes in build."
            } else {
                "Distribution was built with uncommitted changes! Revision does not reflect actual application code!"
            };
            pub const RUST_VERSION: &'static str = __build_info::RUST_VERSION;

            pub const fn formatted() -> &'static str {
                shadow_rs::formatcp!(
"
 Version:       {CRATE_VERSION}
 Target:        {BUILD_TARGET}
 Revision:      {REVISION}
 Revision-Date: {REVISION_DATE}
 Build-Date:    {BUILD_DATE}
 Build-Clean:   {BUILD_CLEAN}
 Rust-Version:  {RUST_VERSION}
"
                )
            }

            pub fn formatted_with_banner(banner: &'static str) -> String {
                let info = formatted();
                format!(
"{banner}
{info}"
                )
            }
        }
    }
}
