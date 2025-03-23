#![allow(dead_code)]

pub mod server;

pub use super::*;
use clap::Parser;
use server::ServerSubcommand;
use tracing::{error, info};

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
            Subcommands::Server(command) => {
                let res = server::run(command);

                match res {
                    Ok(_) => {
                        info!("The server was shut down.");
                    }
                    Err(e) => {
                        error!("The server returned the error: {e:?}");
                    }
                }
            }
        }
    }
}
