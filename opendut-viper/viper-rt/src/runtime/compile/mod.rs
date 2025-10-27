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
