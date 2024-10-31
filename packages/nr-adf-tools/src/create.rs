use std::fs::{self, write};
use std::path::PathBuf;

use anyhow::{
    anyhow,
    Result,
};

use nr_adf_lib::disk::{DiskGeometry, DiskType};

/******************************************************************************
 * Create command run
 *****************************************************************************/

 #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum ArgDiskType {
    DD,
    HD,
}

impl Into<DiskType> for ArgDiskType {
    fn into(self) -> DiskType {
        match self {
            Self::DD => DiskType::DoubleDensity,
            Self::HD => DiskType::HighDensity,
        }
    }
}


#[derive(clap::Args)]
pub struct Args {
    /// Output file
    pub output_file_path: PathBuf,

    /// Specify disk type
    #[arg(long, short = 'F', default_value = "dd")]
    pub floppy_disk_type: ArgDiskType,

    /// Overwrite existing file
    #[arg(long, short, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force_overwrite: bool,
}

pub fn run(args: &Args) -> Result<()> {
    let disk_type: DiskType = args.floppy_disk_type.into();
    let disk_geometry = DiskGeometry::from(disk_type);
    let disk_data = vec![0; disk_geometry.size()];

    if args.output_file_path.exists() && !args.force_overwrite {
        return Err(anyhow!("output file already exists!"));
    }

    fs::write(&args.output_file_path, disk_data)?;
    Ok(())
}
