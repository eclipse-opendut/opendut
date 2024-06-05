use std::io;
use clap::{Command, CommandFactory};
use clap_complete::{generate, Generator, Shell};
use crate::Args;

/// Generates shell completion
#[derive(clap::Parser)]
pub struct CompletionsCli {
    /// shell which will be used for generating completions
    #[arg(required = true)]
    shell: Option<Shell>
}

impl CompletionsCli {
    pub async fn execute(self, args: &mut Args, cmd: &mut Command) -> crate::Result<()> {
        if let Some(generator) = args.generator {
            print_completions(generator, cmd);

        } else {
            println!("no shell given to generate completions");
        }

        Ok(())
    }
}


pub fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
