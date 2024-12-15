use std::fs;
use std::path;
use std::time::SystemTime;

use anyhow::Result;

use chrono::prelude::*;

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

pub fn system_time_to_str(
    st: SystemTime,
) -> String {
    DateTime::<Utc>::from(st).to_rfc2822()
}

pub fn run(args: &Args) -> Result<()> {
    let disk_data = fs::read(&args.input_file_path)?;
    let disk = Disk::try_create_with_data(disk_data)?;

    let fs: AmigaDos = disk.into();
    let fs_boot_block = fs.boot_block()?;
    let fs_root_block = fs.root_block()?;

    println!("           Volume name: {}", fs_root_block.volume_name);
    println!("           Volume type: {}, {}, {}",
        fs_boot_block.filesystem_type(),
        fs_boot_block.international_mode(),
        fs_boot_block.cache_mode(),
    );

    println!("Volume alteration date: {}",
        system_time_to_str(fs_root_block.root_alteration_date)
    );
    println!("    Root creation date: {}",
        system_time_to_str(fs_root_block.root_creation_date)
    );
    println!("  Root alteration date: {}",
        system_time_to_str(fs_root_block.root_alteration_date)
    );

    Ok(())
}
