use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::{anyhow, Result};

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
    let fs = AmigaDos::try_from(Rc::new(RefCell::new(disk)))?;

    for entry in fs.read_dir(&args.amiga_input_filepath)? {
        match entry {
            Ok(entry) => {
                println!("{}", entry.path().to_str().unwrap_or(""));
            },
            Err(err) => {
                return Err(anyhow!("{}", err));
            }
        }
    }

    Ok(())
}
