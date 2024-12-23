use std::fs;
use std::path;

use anyhow::{
    anyhow,
    Result,
};

use nr_adf_lib::disk::Disk;

use crate::cli_common::ArgDiskType;

/******************************************************************************
 * Create command run
 *****************************************************************************/
#[derive(clap::Args)]
pub struct Args {
    /// Output file
    pub output_file_path: path::PathBuf,

    /// Specify disk type
    #[arg(long, short = 'F', default_value = "dd")]
    pub floppy_disk_type: ArgDiskType,

    /// Overwrite existing file
    #[arg(long, short, default_value = "false")]
    pub force_overwrite: bool,
}

pub fn run(args: &Args) -> Result<()> {
    let disk = Disk::create(args.floppy_disk_type.into());

    if args.output_file_path.exists() && !args.force_overwrite {
        Err(anyhow!("output file already exists!"))
    } else {
        fs::write(&args.output_file_path, disk.data())?;
        Ok(())
    }
}
