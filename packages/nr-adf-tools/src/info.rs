use std::fs;
use std::path;
use std::time::SystemTime;

use anyhow::Result;

use chrono::prelude::*;

use nr_adf_lib::prelude::*;


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
    let fs_info = fs.info()?;

    println!("           Volume name: {}", fs_info.volume_name);
    println!("           Volume type: {}, {}, {}",
        fs_info.filesystem_type,
        fs_info.international_mode,
        fs_info.cache_mode,
    );

    println!("Volume alteration date: {}",
        system_time_to_str(fs_info.root_alteration_date)
    );
    println!("    Root creation date: {}",
        system_time_to_str(fs_info.root_creation_date)
    );
    println!("  Root alteration date: {}",
        system_time_to_str(fs_info.root_alteration_date)
    );

    Ok(())
}
