use std::path::PathBuf;
use std::process::Command;

use anyhow::anyhow;
use clap::ArgAction;

use crate::core::{OPENDUT_REPO_ROOT, OPENDUT_VM_NAME};
use crate::core::command_ext::TheoCommandExtensions;
use crate::core::project::ProjectRootDir;

/// Create virtual machine for test environment.
#[derive(clap::Parser)]
pub struct VagrantCli {
    #[command(subcommand)]
    pub(crate) task: TaskCli,
}

#[derive(clap::Subcommand)]
pub enum TaskCli {
    /// Start virtual machine.
    #[command(alias = "start")]
    Up,
    /// Provision virtual machine.
    Provision {
        /// Install desktop to virtual machine
        #[arg(long, short, action = ArgAction::SetTrue)]
        desktop: bool,
    },
    /// Connect to virtual machine via SSH.
    Ssh,
    /// Stop virtual machine.
    #[command(alias = "stop")]
    Halt,
    /// Destroy virtual machine and update the box.
    Destroy,
    /// Reload virtual machine.
    #[command(alias = "restart")]
    Reload,
    /// Get status of virtual machine.
    Status,
    /// Run Firefox remotely on the virtual machine (x11forwarding).
    Firefox,
    /// Alternatives to run Firefox remotely.
    FirefoxRemote,
    /// Run arbitrary Vagrant command.
    #[command(about = "Run arbitrary vagrant command.")]
    Other,
}


impl VagrantCli {
    pub(crate) fn default_handling(&self) -> crate::Result {
        if running_in_opendut_vm() {
            return Err(anyhow!("Command should not be run within the virtual machine."));
        }

        match self.task {
            TaskCli::Up => {
                Command::vagrant().arg("up").run();
            }
            TaskCli::Provision { desktop } => {
                if desktop {
                    Command::vagrant().arg("provision")
                        .env("ANSIBLE_SKIP_TAGS", "")  // remove skip tags
                        .run();
                } else {
                    Command::vagrant().arg("provision").run();
                }
            }
            TaskCli::Ssh => {
                Command::vagrant().arg("ssh").run();
            }
            TaskCli::Halt => {
                Command::vagrant().arg("halt").run();
            }
            TaskCli::Destroy => {
                Command::vagrant().arg("destroy").run();
                // always update the vagrant box once it is destroyed
                Command::vagrant().arg("box").arg("update").run_requiring_success()?;

                println!("\nVagrant box cleanup notes");
                Command::vagrant().arg("box").arg("outdated").run_requiring_success()?;
                println!("\nListing all vagrant boxes:");
                Command::vagrant().arg("box").arg("list").run_requiring_success()?;
                println!("\nNOTE: You may want to delete old boxes with the following command: ");
                println!("  vagrant box remove ubuntu/jammy64 --box-version <X.Y.Z>");
            }
            TaskCli::Reload => {
                Command::vagrant().arg("reload").run();
            }
            TaskCli::Status => {
                Command::vagrant().arg("status").run();
            }
            TaskCli::Other => {
                let project_root = PathBuf::project_path_buf();
                let vagrant_file_path = project_root.join(".ci/docker/Vagrantfile");
                let vagrant_dot_file_path = project_root.join(".vagrant");
                println!("# export the following environment variables to run vagrant commands");
                println!("# Be sure to run the commands from the repository root");
                println!("export {}={:?}", OPENDUT_REPO_ROOT, project_root.into_os_string());
                println!("export VAGRANT_DOTFILE_PATH={:?}", vagrant_dot_file_path.into_os_string());
                println!("export VAGRANT_VAGRANTFILE={:?}", vagrant_file_path.into_os_string());
                println!("# then run 'vagrant <other-command>'")
            }
            TaskCli::Firefox => {
                match home::home_dir() {
                    Some(path) => {
                        let ssh_id_path: String = path.join(".ssh").join("id_rsa").into_os_string().into_string().expect("Could not determine home directory.");
                        Command::new("ssh").arg("-X").arg("-i").arg(ssh_id_path).arg("vagrant@192.168.56.10").arg("export XAUTHORITY=$HOME/.Xauthority; /usr/bin/firefox").run();
                    }
                    None => println!("Impossible to get your home dir!"),
                }
            }

            TaskCli::FirefoxRemote => {
                println!("FirefoxRemote session with xpra: See https://github.com/Xpra-org/xpra/");
                println!("Use xpra: \"xpra start ssh://vagrant@192.168.56.10/ --start=firefox\"");
                println!("Old school VNC session \"vncviewer 192.168.56.10\"");
            }
        }
        Ok(())
    }
}

pub fn running_in_opendut_vm() -> bool {
    let hostname = Command::new("hostname")
        .output()
        .unwrap_or_else(|cause| panic!("Failed to execute hostname. {}", cause));
    let hostname = String::from_utf8(hostname.stdout).expect("Could not determine hostname!");
    hostname.trim().contains(OPENDUT_VM_NAME)
}
