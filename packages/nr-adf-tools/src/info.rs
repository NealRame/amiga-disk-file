use std::cell::RefCell;
use std::fs;
use std::path;
use std::rc::Rc;
use std::time::SystemTime;

use anyhow::Result;

use chrono::prelude::*;

use nr_adf_lib::prelude::*;


/******************************************************************************
 * Format command run
 *****************************************************************************/
#[derive(clap::Args)]
pub struct Args {
    /// Path to an Amiga disk file
    pub amiga_disk_filepath: path::PathBuf,
}

pub fn system_time_to_str(
    st: SystemTime,
) -> String {
    DateTime::<Utc>::from(st).to_rfc2822()
}

pub fn run(args: &Args) -> Result<()> {
    let disk_data = fs::read(&args.amiga_disk_filepath)?;
    let disk = Disk::try_create_with_data(disk_data)?;

    let fs: AmigaDos = Rc::new(RefCell::new(disk)).try_into()?;
    let fs_info = fs.info()?;

    println!("Volume name: {}", fs_info.volume_name);
    println!("Volume type: {}, {}, {}",
        fs_info.filesystem_type,
        fs_info.international_mode,
        fs_info.cache_mode,
    );

    println!();

    println!("Volume alteration date: {}",
        system_time_to_str(fs_info.root_alteration_date)
    );
    println!("    Root creation date: {}",
        system_time_to_str(fs_info.root_creation_date)
    );
    println!("  Root alteration date: {}",
        system_time_to_str(fs_info.root_alteration_date)
    );

    let total_block_count = fs_info.total_block_count;
    let total_size = fs_info.total_size;

    let free_block_count = fs_info.free_block_count;
    let free_size = fs_info.free_size;

    let used_block_count = total_block_count - free_block_count;
    let used_size = total_size - free_size;

    println!();
    println!("Total: {:>8} {:>10}",
        total_block_count,
        total_size,
    );
    println!(" Used: {:>8} {:>10} {:>6.2}%",
        used_block_count,
        used_size,
        100.*((used_block_count as f32)/(total_block_count as f32)),
    );
    println!(" Free: {:>8} {:>10} {:>6.2}%",
        free_block_count,
        free_size,
        100.*((free_block_count as f32)/(total_block_count as f32)),
    );

    Ok(())
}
