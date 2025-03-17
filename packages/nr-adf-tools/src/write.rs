use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::{
    anyhow,
    Result
};

use promptly::prompt_default;

use nr_adf_lib::prelude::*;


fn confirm_overwrite(args: &Args) -> Result<bool> {
    if args.force {
        Ok(true)
    } else {
        prompt_default(
            "Output file already exists. Would you like do overwrite it?",
            false,
        ).map_err(|err| anyhow!(err.to_string()))
    }
}

/******************************************************************************
 * Format command run
 *****************************************************************************/
#[derive(clap::Args)]
pub struct Args {
    /// Path to an Amiga disk file
    amiga_disk_filepath: PathBuf,

    /// Path to a file into the host filesystem
    host_input_filepath: PathBuf,

    /// Path to a file into the Amiga filesystem
    amiga_output_filepath: PathBuf,

    /// If destination file already exists, force overwriting it
    #[arg(short, long)]
    force: bool,
}

pub fn run(args: &Args) -> Result<()> {
    let data = std::fs::read(&args.host_input_filepath)?;
    let disk_data = std::fs::read(&args.amiga_disk_filepath)?;
    let disk = Disk::try_create_with_data(disk_data)?;
    let fs = AmigaDos::try_from(Rc::new(RefCell::new(disk)))?;

    if !fs.exists(&args.amiga_output_filepath)? || confirm_overwrite(args)? {
        fs.write(&args.amiga_output_filepath, data)?;
    }

    fs.dump(&args.amiga_disk_filepath)?;

    Ok(())
}
