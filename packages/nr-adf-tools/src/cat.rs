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

    let mut file = fs.open(&args.path, 0|FileMode::Read)?;
    let mut buf = [0; 16];

    loop {
        match file.read(&mut buf) {
            Ok(count) if count > 0 => {
                for b in &buf[..count] {
                    print!(" {:0>2x}", b);
                }
            },
            _ => break
        }
    }

    Ok(())
}
