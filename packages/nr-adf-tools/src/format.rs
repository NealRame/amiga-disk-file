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

    /// Volune name
    volume_name: String,

    /// Specify the file system type
    #[arg(short = 't', long, default_value = "ofs")]
    filesystem_type: FilesystemType,
}

pub fn run(args: &Args) -> Result<()> {
    let disk = read_disk_file(&args.disk_file_path)?;
    let fs =
        AmigaDosFormater::default()
            .width_filesystem_type(args.filesystem_type)
            .format(disk, &args.volume_name)?;

    write_disk_file(&args.disk_file_path, fs.disk())?;
    Ok(())
}
