#![cfg(feature = "containers")]

use googletest::prelude::*;
use indoc::indoc;
use viper_containers::{CreateContainerConfig, MockClientInvocation, VolumeMountConfig};
use viper_rt::containers::ContainerRuntime;
use viper_rt::events::emitter;
use viper_rt::run::{ParameterBindings, Report};
use viper_rt::source::loaders::EmbeddedSourceLoader;
use viper_rt::source::Source;
use viper_rt::ViperRuntime;

#[tokio::test]
async fn test_container_api() -> Result<()> {

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let (mock_handle, container_runtime) = ContainerRuntime::new_mock();

    let runtime = ViperRuntime::builder()
        .with_source_loader(EmbeddedSourceLoader)
        .with_container_runtime(container_runtime)
        .build()?;

    let (_, _, suite) = runtime.compile(&Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import *

            class MyTestCase(unittest.TestCase):
                def test_create_container(self):
                    self.container.create(
                        "docker.io/library/alpine:latest",
                        ["Hello World"],
                        entrypoint=["echo"],
                        env = ["DEBUG=true"],
                        tty = True,
                        open_stdin = True,
                        name="AlpineContainer",
                        user="1000",
                        volumes=["a:b", "c:d"],
                        network="host"
                    )

                def test_start_container(self):
                    self.container.start("ContainerToStart")

                def test_stop_container(self):
                    self.container.stop("ContainerToStop")
                    
                def test_run_container(self):
                    self.container.run(
                       "docker.io/library/busybox:latest",
                        ["Hello World"],
                        entrypoint=["echo"],
                        env = ["DEBUG=true"],
                        tty = True,
                        open_stdin = True,
                        name="ContainerToRun",
                        user="1000",
                        volumes=["a:b", "c:d"],
                        network="host"
                    )

                def test_remove_container(self):
                    self.container.remove("ContainerToRemove")

                def test_wait_container(self):
                    self.container.wait("ContainerToWaitFor")
                    
                def test_list_containers(self):
                    self.container.list()

                def test_inspect_container(self):
                    self.container.inspect("ContainerToInspect")

                def test_log_container(self):
                    self.container.log("ContainerToLog")

                def test_pull_image(self):
                    self.container.image.pull("docker.io/library/alpine:latest")

                def test_remove_image(self):
                    self.container.image.remove("docker.io/library/alpine:latest")
                    
                def test_list_images(self):
                    self.container.image.list()
        "#)
    ), &mut emitter::drain()).await?.split();

    let run = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;
    let invocations = mock_handle.invocations();

    assert_that!(run.is_success(), eq(true));
    assert_that!(invocations, container_eq([
        MockClientInvocation::CreateContainer {
            config: CreateContainerConfig {
                name: Some(String::from("AlpineContainer")),
                image: String::from("docker.io/library/alpine:latest"),
                command: Some(vec![String::from("Hello World")]),
                entrypoint: Some(vec![String::from("echo")]),
                env: Some(vec![String::from("DEBUG=true")]),
                tty: Some(true),
                open_stdin: Some(true),
                user: Some(String::from("1000")),
                volumes: vec![
                    VolumeMountConfig { src: String::from("a"), dst: String::from("b")},
                    VolumeMountConfig { src: String::from("c"), dst: String::from("d")},
                ],
                network: Some(String::from("host")),
            }
        },
        MockClientInvocation::StartContainer { name: String::from("ContainerToStart") },
        MockClientInvocation::StopContainer { name: String::from("ContainerToStop") },
        
        // Test Docker run
        MockClientInvocation::PullImage { image: String::from("docker.io/library/busybox:latest") },
        MockClientInvocation::CreateContainer {
            config: CreateContainerConfig {
                name: Some(String::from("ContainerToRun")),
                image: String::from("docker.io/library/busybox:latest"),
                command: Some(vec![String::from("Hello World")]),
                entrypoint: Some(vec![String::from("echo")]),
                env: Some(vec![String::from("DEBUG=true")]),
                tty: Some(true),
                open_stdin: Some(true),
                user: Some(String::from("1000")),
                volumes: vec![
                    VolumeMountConfig { src: String::from("a"), dst: String::from("b")},
                    VolumeMountConfig { src: String::from("c"), dst: String::from("d")},
                ],
                network: Some(String::from("host")),
            }
        },
        MockClientInvocation::StartContainer { name: String::from("ContainerToRun") },

        MockClientInvocation::RemoveContainer { name: String::from("ContainerToRemove") },
        MockClientInvocation::WaitContainer { name: String::from("ContainerToWaitFor") },
        MockClientInvocation::ListContainers,
        MockClientInvocation::InspectContainer { name: String::from("ContainerToInspect") },
        MockClientInvocation::LogContainer { name: String::from("ContainerToLog") },
        MockClientInvocation::PullImage { image: String::from("docker.io/library/alpine:latest") },
        MockClientInvocation::RemoveImage { image: String::from("docker.io/library/alpine:latest") },
        MockClientInvocation::ListImages,
    ]));

    Ok(())
}
