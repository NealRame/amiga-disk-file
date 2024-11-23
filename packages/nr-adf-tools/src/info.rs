use std::fs;
use std::path;

use anyhow::Result;

use nr_adf_lib::disk::Disk;
use nr_adf_lib::fs::amiga_dos::AmigaDos;


/******************************************************************************
 * Format command run
 *****************************************************************************/
#[derive(clap::Args)]
pub struct Args {
    /// Input file
    pub input_file_path: path::PathBuf,
}

pub fn run(args: &Args) -> Result<()> {
    let disk_data = fs::read(&args.input_file_path)?;
    let disk = Disk::try_create_with_data(disk_data)?;

    let fs = AmigaDos::try_from(&disk)?;

    println!("Disk type: {}, {}, {}",
        fs.boot_block.filesystem_type(),
        fs.boot_block.international_mode(),
        fs.boot_block.cache_mode(),
    );

    Ok(())
}
