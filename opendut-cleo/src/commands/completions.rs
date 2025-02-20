use std::io;

use clap::Command;
use clap_complete::{generate, Generator};

pub fn print_completions<G: Generator>(generator: G, command: &mut Command) {
    generate(generator, command, command.get_name().to_string(), &mut io::stdout());
}
