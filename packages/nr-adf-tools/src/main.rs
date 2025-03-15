mod cli_common;

mod cat;
mod read;
mod create;
mod format;
mod info;
mod ls;
mod mkdir;


use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new Amiga disk file
    Create(create::Args),
    /// Format a given Amiga disk file
    Format(format::Args),
    /// Get info about a given Amiga disk file
    Info(info::Args),
    /// Cat a file from a given Amiga disk file
    Cat(cat::Args),
    /// List files from a given Amiga disk file
    Ls(ls::Args),
    /// Creates directories named as operands, in the order specified
    Mkdir(mkdir::Args),
    /// Read a file from a given Amiga disk file
    Read(read::Args),
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
        Commands::Mkdir(args) => mkdir::run(args),
        Commands::Read(args) => read::run(args),
    };

    if let Err(err) = res {
        println!("Error: {}", err);
        std::process::exit(1);
    }
}
