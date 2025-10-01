use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;
use anyhow::anyhow;

use crate::core::OPENDUT_REPO_ROOT;
use crate::core::project::ProjectRootDir;

pub(crate) trait TheoCommandExtensions {
    fn vagrant() -> Self;
    fn run_requiring_success(&mut self) -> crate::Result;
    fn run(&mut self);
}

impl TheoCommandExtensions for Command {
    fn vagrant() -> Self {
        // in case git is not present, or command is run outside of repository root, set environment variable
        let project_root_override = env::var(OPENDUT_REPO_ROOT)
            .unwrap_or_else(|_| PathBuf::project_path_buf().into_os_string().into_string().expect("Could not determine project root directory."));

        let mut command = Command::new("vagrant");
        command
            .current_dir(project_root_override.clone())
            .env("VAGRANT_DOTFILE_PATH", ".vagrant")
            .env("VAGRANT_VAGRANTFILE", ".ci/deploy/opendut-vm/Vagrantfile")
            .env(OPENDUT_REPO_ROOT, project_root_override);
        command
    }

    fn run_requiring_success(&mut self) -> crate::Result {
        let status = self.status()
            .unwrap_or_else(|cause| panic!("Error while running command: '{self:?}'\n  {cause}"));

        if status.success() {
            Ok(())
        } else {
            let mut error = format!("Error while running command: '{self:?}'");
            if let Some(status) = &status.code() {
                error += format!("\n  Exited with status code {status}.\n").as_ref();
            }
            Err(anyhow!(error))
        }
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
                    println!("Error terminating child: {error}");
                }
            }
        } else {
            println!("Failed to execute '{self:?}'.");
        }
    }
}
