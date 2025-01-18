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
    /// Disk file
    disk_file_path: PathBuf,

    /// Volune name
    volume_name: String,

    /// Enable/Disable cache mode ["on", "off"]
    #[arg(short = 'c', long = "cache-mode", default_value = "off")]
    cache_mode: CacheMode,

    /// Enable/Disable international mode ["on", "off"]
    #[arg(short = 'i', long = "international-mode", default_value = "off")]
    international_mode: InternationalMode,

    /// Specify the file system type
    #[arg(short = 't', long, default_value = "ofs")]
    filesystem_type: FilesystemType,
}

pub fn run(args: &Args) -> Result<()> {
    let disk = Disk::try_create_with_data(fs::read(&args.disk_file_path)?)?;

    AmigaDosFormater::default()
        .with_cache_mode(args.cache_mode)
        .with_international_mode(args.international_mode)
        .with_filesystem_type(args.filesystem_type)
        .format(
            Rc::new(RefCell::new(disk)),
            &args.volume_name,
        )?
        .dump(&args.disk_file_path)?;

    Ok(())
}
