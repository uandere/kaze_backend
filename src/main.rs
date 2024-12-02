use clap::Parser;

use kaze_backend::commands::Cli;

fn main() {
    let command = Cli::parse();
    command.subcommand.run();
}