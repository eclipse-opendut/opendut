mod emit;
mod inspect;
mod prepare;
mod py;

use crate::compile::{Compilation, CompileEvent, IdentifierFilter};
use crate::runtime::compile::inspect::inspect;
use crate::runtime::compile::prepare::prepare_source_code;
use crate::runtime::compile::py::{compile_source_code, create_interpreter};
use crate::runtime::ctx::Context;
use crate::runtime::emitter::EventEmitter;
use crate::runtime::types::compile::error::{CompilationError, CompileResult};
use crate::source::Source;
use tracing::{debug, error, info};
use crate::runtime::types::compile::filter::FilterError;

pub async fn compile_tree<Emitter>(
    sources: Vec<(Source, Emitter)>,
    context: &Context,
    identifier_filter: &IdentifierFilter
) -> Result<Vec<CompileResult<Compilation>>, FilterError>
where Emitter: EventEmitter<CompileEvent> + Send + 'static,
{
    let mut compilations = Vec::new();

    for (source, mut emitter) in sources {
        if identifier_filter.matches_suite(&source.identifier) {
            let compilation = compile(&source, context, &mut emitter, identifier_filter).await;
            compilations.push(compilation);
        }
    }

    if let Some(suite_identifier_filter) = &identifier_filter.suite_identifier
    && compilations.is_empty() {
        return Err(FilterError::new_test_suite_not_found_error(suite_identifier_filter.to_string()));
    }

    Ok(compilations)
}

pub async fn compile(
    source: &Source,
    context: &Context,
    emitter: &mut dyn EventEmitter<CompileEvent>,
    identifier_filter: &IdentifierFilter,
) -> CompileResult<Compilation> {

    debug!("Compiling test suite '{}'.", source.identifier);

    emit::compilation_started(emitter, source).await?;

    let compilation = compile_source(source, context, identifier_filter).await;

    match &compilation {
        Ok(compilation) => {
            info!("Successfully compiling test suite '{}'.", source.identifier);
            emit::compilation_passed(emitter, source, compilation).await?;
        }
        Err(_) => {
            error!("Failed to compile test suite '{}'!", source.identifier);
            emit::compilation_failed(emitter, source).await?;
        },
    }

    compilation
}

pub async fn compile_source(
    source: &Source,
    context: &Context,
    identifier_filter: &IdentifierFilter,
) -> CompileResult<Compilation> {

    let source_code = prepare_source_code(source, context).await?;
    let interpreter = create_interpreter();
    let py_module = compile_source_code(&source_code, &interpreter)?;
    let (metadata, parameters, suite) = inspect(source_code, py_module, interpreter, identifier_filter)
        .map_err(|error| CompilationError::new_inspection_failure_error(source, error))?;

    Ok(Compilation::new(metadata, parameters, suite))
}
