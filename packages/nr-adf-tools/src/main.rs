mod cli_common;

mod create;
mod format;
mod info;

use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    Create(create::Args),
    Format(format::Args),
    Info(info::Args),
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
    };

    if let Err(err) = res {
        println!("Error: {}", err);
        std::process::exit(1);
    }
}
