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
    /// Input file
    pub input_file_path: path::PathBuf,

    /// path
    pub path: path::PathBuf,
}

pub fn run(args: &Args) -> Result<()> {
    let disk_data = fs::read(&args.input_file_path)?;
    let disk = Disk::try_create_with_data(disk_data)?;
    let fs: AmigaDos = disk.into();

    let mut file = fs.open(&args.path, 0|FileMode::Read)?;
    let mut buf = [0; 16];
    let mut out = stdout();

    loop {
        let count = file.read(&mut buf)?;
        if count > 0 {
            out.write(&buf[..count])?;
        } else {
            break
        }
    }

    Ok(())
}
