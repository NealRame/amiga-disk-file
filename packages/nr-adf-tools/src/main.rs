mod cli_common;

mod cat;
mod create;
mod format;
mod info;
mod ls;

use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new disk file
    Create(create::Args),
    /// Format a given disk file
    Format(format::Args),
    /// Get info about a given disk file
    Info(info::Args),
    /// Cat a file
    Cat(cat::Args),
    /// List files
    Ls(ls::Args),
}

#[derive(Parser)]
#[command(author, about, version)]
#[command(propagate_version = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

fn main() {
    let args = Args::parse();

    let res = match &args.command {
        Commands::Create(args) => create::run(args),
        Commands::Format(args) => format::run(args),
        Commands::Info(args) => info::run(args),
        Commands::Cat(args) => cat::run(args),
        Commands::Ls(args) => ls::run(args),
    };

    if let Err(err) = res {
        println!("Error: {}", err);
        std::process::exit(1);
    }
}
