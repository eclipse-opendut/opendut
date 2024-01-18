use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use clap::ArgAction;
use crate::core::{OPENDUT_REPO_ROOT, OPENDUT_VM_NAME};

use crate::core::project::ProjectRootDir;

/// Create virtual machine for test environment.
#[derive(Debug, clap::Parser)]
pub struct VagrantCli {
    #[command(subcommand)]
    pub(crate) task: TaskCli,
}

#[derive(Debug, clap::Subcommand)]
pub enum TaskCli {
    #[command(about = "Start virtual machine.")]
    Up,
    #[command(about = "Provision virtual machine.")]
    Provision {
        /// Install desktop to virtual machine
        #[arg(long, short, action = ArgAction::SetTrue)]
        desktop: bool,
    },
    #[command(about = "Connect to virtual machine via ssh.")]
    Ssh,
    #[command(about = "Stop virtual machine.")]
    Halt,
    #[command(about = "Destroy virtual machine.")]
    Destroy,
    #[command(about = "Run firefox remotely on the virtual machine (x11forwarding).")]
    Firefox,
    #[command(about = "Alternatives to run firefox remotely.")]
    FirefoxRemote,
    #[command(about = "Run arbitrary vagrant command.")]
    Other,
}

trait VagrantCommand {
    fn vagrant() -> Self;
    fn vagrant_common_args(&mut self) -> &mut Self;
    fn run(&mut self);
}

impl VagrantCommand for Command {
    fn vagrant() -> Self {
        Command::new("vagrant")
    }

    fn vagrant_common_args(&mut self) -> &mut Self {
        let project_root_override = env::var(OPENDUT_REPO_ROOT)
            .unwrap_or_else(|_| PathBuf::project_path_buf().into_os_string().into_string().expect("Could not determine project root directory."));

        self.current_dir(PathBuf::project_dir())
            .env("VAGRANT_VAGRANTFILE", ".ci/docker/Vagrantfile")
            .env(OPENDUT_REPO_ROOT, project_root_override)
    }
    fn run(&mut self) {
        if let Ok(mut child) = self.spawn() {
            let should_terminate = Arc::new(AtomicBool::new(false));

            let signal_terminate = should_terminate.clone();
            ctrlc::set_handler(move || {
                signal_terminate.store(true, Ordering::Relaxed);
            }).expect("Error setting Ctrl-C handler");

            while !should_terminate.load(Ordering::Relaxed) {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        println!("exited with: {status}");
                        break;
                    }
                    Ok(None) => {
                        sleep(Duration::from_secs(1));
                    }
                    Err(e) => println!("error attempting to wait: {e}"),
                }
            }
            if should_terminate.load(Ordering::Relaxed) {
                println!("Terminating child process.");
            }
            println!("Waiting for child process to terminate.");
            match child.kill() {
                Ok(_) => {}
                Err(error) => {
                    println!("Error terminating child: {}", error);
                }
            }

        } else {
            println!("Failed to execute '{:?}'.", self);
        }
    }
}


impl VagrantCli {
    pub(crate) fn default_handling(&self) {
        if running_in_opendut_vm() {
            panic!("Command should not be run within the vagrant vm.")
        }

        match self.task {
            TaskCli::Up => {
                Command::vagrant().vagrant_common_args().arg("up").run();
            }
            TaskCli::Provision { desktop } => {
                if desktop {
                    Command::vagrant().vagrant_common_args().arg("provision")
                        .env("ANSIBLE_SKIP_TAGS", "")  // remove skip tags
                        .run();
                } else {
                    Command::vagrant().vagrant_common_args().arg("provision").run();
                }
            }
            TaskCli::Ssh => {
                Command::vagrant().vagrant_common_args().arg("ssh").run();
            }
            TaskCli::Halt => {
                Command::vagrant().vagrant_common_args().arg("halt").run();
            }
            TaskCli::Destroy => {
                Command::vagrant().vagrant_common_args().arg("destroy").run();
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
                    },
                    None => println!("Impossible to get your home dir!"),
                }
            }

            TaskCli::FirefoxRemote => {
                println!("FirefoxRemote session with xpra: See https://github.com/Xpra-org/xpra/");
                println!("Use xpra: \"xpra start ssh://vagrant@192.168.56.10/ --start=firefox\"");
                println!("Old school VNC session \"vncviewer 192.168.56.10\"");
            }
        }
    }
}

pub fn running_in_opendut_vm() -> bool {
    let hostname = Command::new("hostname")
        .output()
        .unwrap_or_else(|cause| panic!("Failed to execute hostname. {}", cause));
    let hostname = String::from_utf8(hostname.stdout).expect("Could not determine hostname!");
    let opendut_vm_name = String::from(OPENDUT_VM_NAME);
    opendut_vm_name.eq(hostname.trim())
}
