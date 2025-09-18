use indoc::indoc;
use std::error::Error;
use std::ops::Not;
use tracing::info;
use viper_rt::compile::ParameterName;
use viper_rt::containers::ContainerRuntime;
use viper_rt::events::emitter;
use viper_rt::run::{BindingValue, Outcome, ParameterBindings, Report};
use viper_rt::source::loaders::EmbeddedSourceLoader;
use viper_rt::source::Source;
use viper_rt::ViperRuntime;

#[tokio::main()]
async fn main() -> Result<(), Box<dyn Error>> {

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Running example.");

    let viper = ViperRuntime::builder()
        .with_source_loader(EmbeddedSourceLoader)
        .with_container_runtime(ContainerRuntime::new_docker()?)
        .build()?;

    let source = Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import *

            print("Initializing test")

            METADATA = metadata.Metadata(
                description = "This is an example"
            )

            MESSAGE = parameters.TextParameter("message")
            MAGIC_NUMBER = parameters.NumberParameter("magic_number")
            ENABLED = parameters.BooleanParameter("enabled")

            DEFAULT_NAME = parameters.TextParameter("name", default="Elmar")
            DEFAULT_PORT = parameters.NumberParameter("port", default=8000)
            DEFAULT_BOOL = parameters.BooleanParameter("bool", default=True)
            
            ECU = parameters.NumberParameter("ecu", display_name="ECU Port", description="The port to use when connecting to the ECU.", default=3000)

            name = "Elmar"

            class MyTestCase(unittest.TestCase):

                @classmethod
                def setUpClass(cls):
                    print("setUpClass called")
                    cls.shared_resource = "Shared resource setup"

                def setUp(self):
                    self.x = self.parameters.get(MAGIC_NUMBER)
                    print("Parameter DEFAULT_NAME: ", self.parameters.get(DEFAULT_NAME))
                    print("Parameter DEFAULT_PORT: ", self.parameters.get(DEFAULT_PORT))
                    print("Parameter DEFAULT_BOOL: ", self.parameters.get(DEFAULT_BOOL))
                    print("Parameter ECU: ", self.parameters.get(ECU))
                    
                def test_example(self):
                    print(self.shared_resource)
                    self.x += 1
                    self.assertEquals(4, self.x)

                def test_assert_1(self):
                    self.x = self.x + 1
                    self.assertEquals(4, self.x)

                def test_container_success(self):
                    if self.parameters.get("enabled"):
                        message = self.parameters.get(MESSAGE)
                        self.container.remove("LsContainerSuccess")
                        self.container.image.pull("docker.io/library/alpine:latest")

                        ls = self.container.create(
                            "docker.io/library/alpine:latest",
                            [f"/"],
                            entrypoint=["ls"],
                            env = ["DEBUG=true"],
                            name="LsContainerSuccess",
                            user="1000"
                        )

                        self.container.start(ls)
                        inspected_container = self.container.inspect(ls)

                        print("Name: " + inspected_container.name)
                        print(f"Args: { inspected_container.args }")
                        print(f"Log: { self.container.log(ls) }")

                        exit_code = inspected_container.state.exit_code
                        self.assertEquals(0, exit_code)

                def test_container_failure(self):
                    self.container.remove("LsContainerFailure")
                    self.container.image.pull("docker.io/library/alpine:latest")

                    ls = self.container.create(
                        "docker.io/library/alpine:latest",
                        [f"/idontexist"],
                        entrypoint=["ls"],
                        env = ["DEBUG=true"],
                        name="LsContainerFailure",
                        user="1000"
                    )

                    self.container.start(ls)
                    inspected_container = self.container.inspect(ls)

                    print("Name: " + inspected_container.name)
                    print(f"Args: { inspected_container.args }")
                    print(f"Log:  { self.container.log(ls) }")

                    exit_code = inspected_container.state.exit_code
                    self.assertNotEquals(0, exit_code)

            class TestBla(unittest.TestCase):
                def test_something(self):
                    pass
        "#)
    );

    let (_, parameters, suite) = viper.compile(&source, &mut emitter::drain()).await?.split();

    let mut bindings = ParameterBindings::from(parameters);

    bindings.bind(&ParameterName::try_from("message")?, BindingValue::TextValue(String::from("Hello ")))?;
    bindings.bind(&ParameterName::try_from("magic_number")?, BindingValue::NumberValue(3))?;
    bindings.bind(&ParameterName::try_from("enabled")?, BindingValue::BooleanValue(true))?;

    let bindings = bindings.complete()?;

    let test_suite_report = viper.run(suite, bindings, &mut emitter::drain()).await?;

    match test_suite_report.outcome() {
        Outcome::Success => println!("All tests passed."),
        Outcome::Failure => println!("At least one test failed."),
    }

    for test_case_report in test_suite_report.cases {
        for test_report in test_case_report.tests {
            println!("Test '{}': {}", test_case_report.name, test_report.outcome);
            for line in test_report.output.iter() {
                let line = line.trim();
                if line.is_empty().not() {
                    println!("\t{line}");
                }
            }
        }
    }

    Ok(())
}
