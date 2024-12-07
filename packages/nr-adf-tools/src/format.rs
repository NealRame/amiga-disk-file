use std::path;

use anyhow::Result;

use crate::cli_common::ArgDiskType;

/******************************************************************************
 * Format command run
 *****************************************************************************/
#[derive(clap::Args)]
pub struct Args {
    /// Output file
    pub input_file_path: path::PathBuf,

    /// Specify disk type
    #[arg(long, short = 'F', default_value = "dd")]
    pub floppy_disk_type: ArgDiskType,
}

pub fn run(_: &Args) -> Result<()> {
    Ok(())
}
