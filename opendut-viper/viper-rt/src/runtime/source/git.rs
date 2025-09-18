use crate::runtime::source::loader::SourceLoaderResult;
use crate::runtime::source::SourceLoader;
use crate::source::Source;

pub struct GitSourceLoader;

#[async_trait::async_trait]
impl SourceLoader for GitSourceLoader {

    fn identifier(&self) -> &str {
        "GitSourceLoader"
    }

    fn supports(&self, _source: &Source) -> bool {
        todo!()
    }

    async fn load(&self, _source: &Source) -> SourceLoaderResult {
        todo!()
    }
}
