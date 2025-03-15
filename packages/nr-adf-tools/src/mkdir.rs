use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::Result;

use nr_adf_lib::prelude::*;


/******************************************************************************
 * Format command run
 *****************************************************************************/
#[derive(clap::Args)]
pub struct Args {
    /// Path to an Amiga disk file
    amiga_disk_filepath: PathBuf,

    /// Path to a directory into the Amiga filesystem
    amiga_directory_filepath: PathBuf,

    /// Create intermediate directories as required.
    #[arg(short, long)]
    parent: bool,
}

pub fn run(args: &Args) -> Result<()> {
    let disk_data = fs::read(&args.amiga_disk_filepath)?;
    let disk = Disk::try_create_with_data(disk_data)?;
    let mut fs = AmigaDos::try_from(Rc::new(RefCell::new(disk)))?;

    if args.parent {
        fs.create_dir_all(&args.amiga_directory_filepath)?;
    } else {
        fs.create_dir(&args.amiga_directory_filepath)?;
    }

    fs.dump(&args.amiga_disk_filepath)?;

    Ok(())
}
