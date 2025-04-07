mod cli_common;

mod cat;
mod create;
mod format;
mod info;
mod ls;
mod mkdir;
mod read;
mod rm;
mod touch;
mod write;


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
    #[command(visible_alias="ls")]
    List(ls::Args),
    /// Creates directories named as operands, in the order specified
    Mkdir(mkdir::Args),
    /// Read a file from a given Amiga disk file
    Read(read::Args),
    /// Remove a file or a directory from a given Amiga disk file
    #[command(visible_alias="rm")]
    Remove(rm::Args),
    /// Change file modification times
    Touch(touch::Args),
    /// Write a file to a given Amiga disk location
    Write(write::Args),
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
        Commands::List(args) => ls::run(args),
        Commands::Mkdir(args) => mkdir::run(args),
        Commands::Read(args) => read::run(args),
        Commands::Remove(args) => rm::run(args),
        Commands::Touch(args) => touch::run(args),
        Commands::Write(args) => write::run(args),
    };

    if let Err(err) = res {
        println!("Error: {}", err);
        std::process::exit(1);
    }
}
