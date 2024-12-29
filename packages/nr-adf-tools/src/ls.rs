use std::fs;
use std::path;

use anyhow::Result;

use nr_adf_lib::prelude::*;


/******************************************************************************
 * Format command run
 *****************************************************************************/
#[derive(clap::Args)]
pub struct Args {
    /// Input file
    pub input_file_path: path::PathBuf,

    /// path
    pub path: path::PathBuf,
}

pub fn run(args: &Args) -> Result<()> {
    let disk_data = fs::read(&args.input_file_path)?;
    let disk = Disk::try_create_with_data(disk_data)?;
    let fs: AmigaDos = disk.into();

    for entry in fs.read_dir(&args.path)? {
        if let Ok(entry) = entry {
            println!("{}", entry.path().to_str().unwrap_or(""));
        }
    }

    Ok(())
}
