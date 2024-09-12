pub mod init;
pub mod plugin_runtime;
mod setup_plugin;

mod constants {
    use std::path::PathBuf;
    use anyhow::anyhow;
    use opendut_util::project;

    pub fn path_in_edgar_distribution() -> anyhow::Result<PathBuf> {
        let path = PathBuf::from("plugins");
        project::make_path_absolute(&path)
            .map_err(|cause| anyhow!("Failed to determine absolute path of the plugin folder in the unpacked EDGAR distribution, which is supposed to be at '{path:?}': {cause}"))
    }
}
