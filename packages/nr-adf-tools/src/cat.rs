use std::fs;
use std::io::{stdout, Write};
use std::path;

use anyhow::Result;

use nr_adf_lib::prelude::*;

/******************************************************************************
 * Format command run
 *****************************************************************************/
#[derive(clap::Args)]
pub struct Args {
    /// Path to an Amiga disk file
    pub amiga_disk_filepath: path::PathBuf,

    /// Path to a file into the Amiga filesystem
    pub amiga_input_filepath: path::PathBuf,
}

pub fn run(args: &Args) -> Result<()> {
    let disk_data = fs::read(&args.amiga_disk_filepath)?;
    let disk = Disk::try_create_with_data(disk_data)?;

    let fs: AmigaDos = disk.into();
    let data = fs.read(&args.amiga_input_filepath)?;

    stdout().write(&data)?;

    Ok(())
}
