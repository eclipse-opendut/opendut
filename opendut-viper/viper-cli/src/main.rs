mod templates;
mod console;
mod parse;
mod param_config;

use std::collections::HashMap;
use clap::Parser;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Write};
use std::ops::Not;
use std::path::absolute;
use futures::{Sink, SinkExt};
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::{PollSendError, PollSender};
use tracing_subscriber::EnvFilter;
use viper_rt::common::TestSuiteIdentifier;
use viper_rt::compile::CompileEvent;
use viper_rt::containers::ContainerRuntime;
use viper_rt::events::emitter;
use viper_rt::run::{ParameterBindings, RunEvent};
use viper_rt::source::loaders::SimpleFileSourceLoader;
use viper_rt::source::Source;
use viper_rt::ViperRuntime;
use crate::console::Event;
use crate::param_config::{IncompleteBindingsError, ParameterToml};
use crate::templates::SCRIPT_PY_TEMPLATE;

#[derive(clap::Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command
}

#[derive(clap::Subcommand)]
enum Command {
    Init,
    Run {
        #[arg(long="params-from-file")]
        params_from_file: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("none"))
        // .with_writer(std::io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();

    match cli.command {
        Command::Init => {
            if let Err(e) = create_project_structure() {
                eprintln!("Error creating project structure: {e}");
                std::process::exit(1);
            }
        }
        Command::Run{ params_from_file } => {
            if let Err(e) = build_and_run(params_from_file).await {
                eprintln!("Error running tests: {e}");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn create_project_structure() -> io::Result<()> {
    fs::create_dir_all("src/viper")?;

    let mut container_py = File::create("src/viper/container.py")?;
    container_py.write_all(viper_py::container::container::PyContainerRuntimeProxy::GENERATED_PYTHON_CODE.as_bytes())?;
    container_py.write_all(viper_py::container::container::PyContainerRuntimeImageProxy::GENERATED_PYTHON_CODE.as_bytes())?;

    let mut file_py = File::create("src/viper/file.py")?;
    file_py.write_all(viper_py::file::file::FileHandler::GENERATED_PYTHON_CODE.as_bytes())?;

    let mut metadata_py = File::create("src/viper/metadata.py")?;
    metadata_py.write_all(viper_py::metadata::metadata::PyMetadata::GENERATED_PYTHON_CODE.as_bytes())?;

    let mut parameter_py = File::create("src/viper/parameter.py")?;
    parameter_py.write_all(viper_py::parameters::parameters::PyParameterDict::GENERATED_PYTHON_CODE.as_bytes())?;

    let mut report_py = File::create("src/viper/report.py")?;
    report_py.write_all(viper_py::report::report::PyReportProperties::GENERATED_PYTHON_CODE.as_bytes())?;

    let mut unittest_py = File::create("src/viper/unittest.py")?;
    unittest_py.write_all(viper_py::unittest::unittest::TestCase::GENERATED_PYTHON_CODE.as_bytes())?;

    let mut script_py = File::create("src/script.py")?;
    script_py.write_all(SCRIPT_PY_TEMPLATE.as_bytes())?;

    println!("Python files created successfully!");
    Ok(())
}

async fn build_and_run(params_from_file: Option<String>) -> Result<(), Box<dyn Error>> {

    let runtime = ViperRuntime::builder()
        .with_source_loader(SimpleFileSourceLoader)
        .with_container_runtime(ContainerRuntime::new_docker()?)
        .build()?;

    let files = fs::read_dir("./src").unwrap();

    let render_task = {
        let (sender, receiver) = tokio::sync::mpsc::channel::<Event>(64);

        let render_task = tokio::spawn(async move {
            let _ = console::render(ReceiverStream::new(receiver)).await;
        });

        let mut test_suites = Vec::new();

        let parameter_toml = ParameterToml::load(&params_from_file)?;

        let mut bindings_map = HashMap::new();

        for file in files {
            let Ok(file) = file else {
                panic!("No such file or directory!");
            };

            let path = absolute(file.path())?;

            let Some(file_name) = path.file_name() else {
                panic!("Path must be a valid file!");
            };

            let file_name = file_name.to_string_lossy();

            if file_name == "viper" || file_name.ends_with(".py").not() {
                continue;
            }

            let test_suite_identifier = TestSuiteIdentifier::try_from(file_name)?;

            let source = Source::try_from_path(test_suite_identifier, &path)?;

            let mut emitter = emitter::sink(new_compile_event_sink(&sender));

            let (_, descriptors, suite) = runtime.compile(&source, &mut emitter).await?.split();

            let mut bindings = ParameterBindings::from(Clone::clone(&descriptors));

            if params_from_file.is_some() {
                parameter_toml.bind_parameters_for_suite(suite.name(), &mut bindings)?;
            }

            let completed_bindings_result = bindings.complete();

            let completed_bindings = match completed_bindings_result {
                Ok(completed_bindings) => {
                    completed_bindings
                }
                Err(err) => {
                    return Err(IncompleteBindingsError {
                        suite: suite.name().to_string(),
                        cause: err
                    }.into());
                }
            };

            bindings_map.insert(suite.name().to_string(), completed_bindings);
            test_suites.push(suite);
        }

        sender.send(Event::RenderEvent).await?;

        let mut emitter = emitter::sink(new_run_event_sink(&sender));
        for suite in test_suites {
            let bindings = bindings_map.remove(suite.name());
            if let Some(complete_bindings) = bindings {

                runtime.run(suite, complete_bindings, &mut emitter).await?;
            }
        }

        render_task
    };

    render_task.await?;
    Ok(())
}

fn new_run_event_sink(sender: &Sender<Event>) -> impl Sink<RunEvent, Error = PollSendError<Event>> + Unpin {
    Box::pin(
        PollSender::new(Clone::clone(sender))
            .with(|event| async { Ok(Event::RunEvent(event)) })
    )
}

fn new_compile_event_sink(sender: &Sender<Event>) -> impl Sink<CompileEvent, Error = PollSendError<Event>> + Unpin {
    Box::pin(
        PollSender::new(Clone::clone(sender))
            .with(|event| async { Ok(Event::CompileEvent(event))})
    )
}
