pub struct Context {
    #[cfg(feature = "compile")]
    pub source_loaders: Vec<Box<dyn crate::runtime::source::SourceLoader>>,
    #[cfg(feature = "containers")]
    pub container_runtime: Option<opendut_viper_containers::ContainerRuntime>,
}
