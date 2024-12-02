#![allow(dead_code)]

pub mod subcommands;

pub use super::*;
use clap::Parser;
use subcommands::server::ServerSubcommand;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Subcommands,
}

#[derive(Parser)]
pub enum Subcommands {
    Server(ServerSubcommand),
}

impl Subcommands {
    pub fn run(self) {
        match self {
            Subcommands::Server(command) => subcommands::server::run(command),
        }
    }
}