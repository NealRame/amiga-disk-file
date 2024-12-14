use std::fs;
use std::path;

use anyhow::Result;

use nr_adf_lib::prelude::*;


fn read_disk_file(
    input_disk_file_path: &path::Path,
) -> Result<Disk> {
    Ok(Disk::try_create_with_data(fs::read(input_disk_file_path)?)?)
}

fn write_disk_file(
    output_disk_file_path: &path::Path,
    disk: &Disk,
) -> Result<()> {
    Ok(fs::write(output_disk_file_path, disk.data())?)
}

/******************************************************************************
 * Format command run
 *****************************************************************************/
#[derive(clap::Args)]
pub struct Args {
    /// Disk file
    disk_file_path: path::PathBuf,

    /// Specify the file system type
    #[arg(short = 't', long, default_value = "ofs")]
    filesystem_type: FilesystemType,
}

pub fn run(args: &Args) -> Result<()> {
    let mut disk = read_disk_file(&args.disk_file_path)?;

    let boot_block =
        BootBlockBuilder::new(disk.disk_type())
            .width_filesystem_type(args.filesystem_type)
            .build();

    boot_block.try_write_to_disk(&mut disk)?;

    write_disk_file(&args.disk_file_path, &disk)?;

    Ok(())
}
