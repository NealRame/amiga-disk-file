mod cli_common;

mod create;
mod info;
mod format;

use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    Create(create::Args),
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
        Commands::Info(args) => info::run(args),
    };

    if let Err(err) = res {
        println!("Error: {}", err);
        std::process::exit(1);
    }
}
