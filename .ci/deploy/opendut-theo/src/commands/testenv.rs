use crate::commands::edgar::TestEdgarCli;
use crate::core::docker::command::DockerCommand;
use crate::core::docker::show_error_if_unhealthy_containers_were_found;
use crate::core::project::load_theo_environment_variables;
use crate::core::{localenv, TestenvMode};
use clap::ArgAction;

/// Build and start test environment.
#[derive(clap::Parser)]
pub struct TestenvCli {
    #[command(subcommand)]
    pub(crate) task: TaskCli,
}

#[derive(clap::Subcommand)]
pub enum TaskCli {
    /// Build Docker containers.
    Build,
    /// Provision secrets for localenv.
    Provision,
    /// Start test environment.
    #[command(alias = "up")]
    Start {
        /// Expose firefox container port (3000), or set OPENDUT_EXPOSE_PORTS=true
        #[arg(long, short, action = ArgAction::SetTrue)]
        expose: bool,

        #[arg(long, action = ArgAction::SetTrue)]
        skip_firefox: bool,

        /// Skip telemetry (set OPENDUT_LOCALENV_TELEMETRY_ENABLED=0)
        #[arg(long, action = ArgAction::SetTrue)]
        skip_telemetry: bool,
    },
    /// Stop test environment.
    Stop,
    /// Show Docker network.
    Network,
    /// Destroy test environment.
    Destroy,
    /// Run EDGAR cluster creation.
    Edgar(TestEdgarCli),
}

impl TestenvCli {
    pub(crate) fn default_handling(self) -> crate::Result {
        load_theo_environment_variables();
        DockerCommand::new().docker_checks()?;

        let run_carl_dist_mode = TestenvMode::CarlDistribution;
        match self.task {
            TaskCli::Build => {
                localenv::build_localenv_containers(&run_carl_dist_mode)?;
            }
            TaskCli::Provision => {
                localenv::provision_localenv_secrets()?;
            }
            TaskCli::Start { expose, skip_firefox, skip_telemetry } => {
                localenv::start(skip_telemetry, skip_firefox, expose, &run_carl_dist_mode)?;
            }
            TaskCli::Stop => {
                localenv::stop()?;
            }
            TaskCli::Network => {
                crate::core::network::docker_inspect_network()?;
                show_error_if_unhealthy_containers_were_found()?;
            }
            TaskCli::Destroy => {
                localenv::destroy()?;
            }
            TaskCli::Edgar(cli) => {
                cli.default_handling()?;
            }
        }
        Ok(())
    }


}

