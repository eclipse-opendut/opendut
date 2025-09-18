use crate::runtime::compile::CompileResult;
use crate::runtime::ctx::Context;
use crate::runtime::types::compile::code::{ApiVersion, SourceCode};
use crate::runtime::types::compile::error::CompilationError;
use crate::runtime::types::source::error::InvalidSourceError;
use crate::source::Source;
use std::ops::Not;

pub async fn prepare_source_code(source: &Source, context: &Context) -> CompileResult<SourceCode> {

    let content = load_source(source, context).await?;
    let version = parse_version(&content)
        .map_err(|error| CompilationError::new_invalid_source_error(source, error))?;

    Ok(SourceCode {
        identifier: Clone::clone(&source.identifier),
        code: content,
        version,
    })
}

pub async fn load_source(source: &Source, context: &Context) -> CompileResult<String> {
    let loader = context.source_loaders.iter().find(|loader| loader.supports(source))
        .ok_or_else(|| CompilationError::new_no_suitable_source_loader_error(source))?;
    let code = loader.load(source).await
        .map_err(|error| CompilationError::new_source_loading_failure_error(source, error))?;
    Ok(code)
}

pub fn parse_version(content: &str) -> Result<ApiVersion, InvalidSourceError> {

    let mut lines = content.lines();

    let version = {
        let first_line = lines.next().ok_or(InvalidSourceError::new_empty_source_error())?; // TODO: Relax search for API_VERSION, so that there can be blank lines or other comments above.
        if first_line.starts_with("# VIPER_VERSION").not() {
            return Err(InvalidSourceError::new_missing_viper_version_error()); // TODO: Add more information about the error.
        }
        let version_string = first_line.split('=')
            .nth(1)
            .and_then(|part| part.split('#').next())
            .map(|s| s.trim())
            .ok_or(InvalidSourceError::new_illegal_viper_version_string_error(first_line))?;
        match version_string {
            "1.0" => ApiVersion::V1_0,
            _ => return Err(InvalidSourceError::new_unknown_viper_version_error(version_string)),
        }
    };

    Ok(version)
}
