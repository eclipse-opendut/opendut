/// Defines a function called `log_function`, to pass into [AsyncProcessManager::spawn_process](super::AsyncProcessManager::spawn_process).
///
/// ```no_compile
/// let log_function = create_process_log_function!("cannelloni", extra="fields");
/// ```
///
/// This macro was necessary to introduce, because the [tracing::trace] macro does not allow passing the `target`-field, unless it is a constant or a literal.
/// The target is used for selecting which logs to show, so rather essential.
macro_rules! create_process_log_function {
    ($name:expr) => {
        create_process_log_function!($name, )
    };
    ($name:expr, $($field:ident=$value:expr),*) => {
        move |
            process: &crate::service::process_manager::ProcessLoggingMetadata,
            line: &str
        | {
            ::tracing::trace!(
                target: $name,
                process_name=process.name,
                process_id=process.id,
                process_stream=process.stream,
                $(
                    $field=$value,
                )*
                "{line}"
            );
        }
    };
}
pub(crate) use create_process_log_function;

/// Type alias for the type of closure created by [create_process_log_function].
/// Rust's native `type` aliases don't currently support `impl`, which is needed for specifying the type of a closure.
#[allow(non_snake_case)]
macro_rules! ProcessLogFunction {
    () => {
        impl Fn(&ProcessLoggingMetadata, &str) + Send + Clone + 'static
    }
}
pub(crate) use ProcessLogFunction;


/// Only intended for use in `create_log_function!()` macro.
pub(crate) struct ProcessLoggingMetadata {
    pub(crate) name: String,
    pub(crate) id: Option<u32>,
    pub(crate) stream: &'static str,
}

#[test]
fn should_create_process_log_function() {
    let _ = create_process_log_function!("netbird-service");
    let _ = create_process_log_function!("cannelloni", extra="fields", hello=123);
}
