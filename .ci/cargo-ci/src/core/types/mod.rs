mod arch;
mod package;
pub mod parsing;

pub use arch::Arch;
pub use package::Package;

/// Result generally used for tasks.
/// This does not allow returning data from tasks,
/// as we usually want task results to be written to disk
/// so that these tasks can be called in a modular fashion from automated runners.
pub type Result = anyhow::Result<()>;
