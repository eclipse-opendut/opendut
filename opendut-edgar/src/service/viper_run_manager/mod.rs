use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::warn;
use opendut_model::viper::{TestRunSourceCode, ViperRunId, ViperTestParameters};
use opendut_viper_rt::ViperRuntime;
use opendut_viper_rt::compile::{CompilationError, IdentifierFilter};
use opendut_viper_rt::events::emitter;
use opendut_viper_rt::run::{BindParameterError, IncompleteParameterBindingsError, ParameterBindings, RunError, TestSuiteReport};
use opendut_viper_rt::source::Source;

pub type ViperRunManagerRef = Arc<ViperRunManager>;

pub struct ViperRunManager {
    test_runs: RwLock<HashMap<ViperRunId, JoinHandle<Result<TestSuiteReport, StartTestRunError>>>>,
}

impl Debug for ViperRunManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViperRunManager")
            .finish()
    }
}

impl ViperRunManager {
    pub fn create() -> ViperRunManagerRef {
        Arc::new(Self {
            test_runs: RwLock::new(HashMap::new()),
        })
    }

    pub async fn start_test_run(&self, run_id: ViperRunId, source_code: TestRunSourceCode, parameters: ViperTestParameters) {
        let handle = tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async move {
                #[allow(clippy::needless_update)]
                let viper_runtime = ViperRuntime::default();

                let source = Source::embedded(source_code.inner.code);

                let compilation = viper_runtime.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await
                    .map_err(StartTestRunError::Compilation)?;

                let (_, parameter_descriptors, suite) = compilation.split();

                let mut bindings = ParameterBindings::from(parameter_descriptors);

                for (name, value) in parameters.iter() {
                    if let Some(value) = value {
                        bindings
                            .bind(name, value.clone())
                            .map_err(StartTestRunError::BindParameter)?;
                    }
                }

                let completed_bindings = bindings.complete()
                    .map_err(StartTestRunError::IncompleteParameterBindings)?;

                let report = viper_runtime.run(suite, completed_bindings, &mut emitter::drain()).await
                    .map_err(StartTestRunError::Run)?;

                Ok(report)
            })
        });

        let mut test_runs = self.test_runs.write().await;
        test_runs.insert(run_id, handle);
    }

    pub async fn contains_test_run(&self, run_id: &ViperRunId) -> bool {
        let test_runs = self.test_runs.read().await;
        test_runs.contains_key(run_id)
    }

    pub async fn abort_test_run(&self, run_id: &ViperRunId) {
        let mut test_runs = self.test_runs.write().await;
        let removed_handle = test_runs.remove(run_id);
        match removed_handle {
            Some(handle) => handle.abort(),
            None => {
                warn!("Thread handle in ViperRunManager for test run {run_id} not found");
            }
        }
    }
}

#[derive(Debug, Error)]
enum StartTestRunError {
    #[error(transparent)]
    Compilation(Box<CompilationError>),

    #[error(transparent)]
    Run(Box<RunError>),

    #[error(transparent)]
    BindParameter(BindParameterError),

    #[error(transparent)]
    IncompleteParameterBindings(IncompleteParameterBindingsError),
}
