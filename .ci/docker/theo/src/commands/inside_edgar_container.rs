use clap::Subcommand;
use std::process::Command;

/// Tasks to run inside containers.
#[derive(clap::Parser)]
#[command(hide=true)]
pub struct InsideEdgarContainerCli {
    #[command(subcommand)]
    pub(crate) task: InsideEdgarContainerTaskCli,
}

#[derive(Subcommand)]
pub enum InsideEdgarContainerTaskCli {
    /// Setup inside the leader peer.
    Leader,
    /// Setup inside any member peer.
    Member,
}

pub fn call_managed_sh(task: InsideEdgarContainerTaskCli) -> crate::Result {
    match task {
        InsideEdgarContainerTaskCli::Leader => {
            println!("Setting up leader.");
            Command::new("bash").arg("/opt/managed.sh").arg("leader").status()?;
        }
        InsideEdgarContainerTaskCli::Member => {
            println!("Setting up member.");
            Command::new("bash").arg("/opt/managed.sh").status()?;
        }
    };
    Ok(())
}
