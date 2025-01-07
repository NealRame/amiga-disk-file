use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use nr_adf_lib::prelude::*;


/******************************************************************************
 * Format command run
 *****************************************************************************/
#[derive(clap::Args)]
pub struct Args {
    /// Path to an Amiga disk file
    pub amiga_disk_filepath: PathBuf,

    /// Path to a file into the Amiga filesystem
    pub amiga_input_filepath: PathBuf,
}

pub fn run(args: &Args) -> Result<()> {
    let disk_data = fs::read(&args.amiga_disk_filepath)?;
    let disk = Disk::try_create_with_data(disk_data)?;
    let fs: AmigaDos = disk.try_into()?;

    for entry in fs.read_dir(&args.amiga_input_filepath)? {
        if let Ok(entry) = entry {
            println!("{}", entry.path().to_str().unwrap_or(""));
        }
    }

    Ok(())
}
