use crate::runtime::source::SourceLoader;

pub struct Context {
    pub source_loaders: Vec<Box<dyn SourceLoader>>,
    #[cfg(feature = "containers")]
    pub container_runtime: Option<viper_containers::ContainerRuntime>,
}
