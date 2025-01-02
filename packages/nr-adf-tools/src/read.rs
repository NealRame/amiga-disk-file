use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{
    anyhow,
    Result
};

use promptly::prompt_default;

use nr_adf_lib::prelude::*;


fn get_output_filepath(args: &Args) -> Result<PathBuf> {
    if let Some(host_output_filepath) = &args.host_output_filepath {
        return Ok(host_output_filepath.clone());
    }

    if let Some(file_name) = args.amiga_disk_filepath.file_name() {
        Ok(PathBuf::from(file_name))
    } else {
        Err(anyhow!("Invalid output file path"))
    }
}

fn confirm_overwrite(args: &Args) -> Result<bool> {
    if args.force {
        Ok(true)
    } else {
        prompt_default(
            "Output file already exists. Would you like do overwrite it?",
            false,
        ).map_err(|err| anyhow!(err.to_string()))
    }
}

fn get_output_file(args: &Args) -> Result<Option<fs::File>> {
    let output_filepath = get_output_filepath(args)?;

    if !fs::exists(&output_filepath)? || confirm_overwrite(args)? {
        let file = fs::File::create(output_filepath)?;
        Ok(Some(file))
    } else {
        Ok(None)
    }
}

/******************************************************************************
 * Format command run
 *****************************************************************************/
#[derive(clap::Args)]
pub struct Args {
    /// Path to an Amiga disk file
    pub amiga_disk_filepath: PathBuf,

    /// Path to a file into the Amiga filesystem
    pub amiga_input_filepath: PathBuf,

    /// Path to a file into the host filesystem
    pub host_output_filepath: Option<PathBuf>,

    /// If output file already exists, force overwriting it
    #[arg(short, long)]
    force: bool,
}

pub fn run(args: &Args) -> Result<()> {
    let disk_data = fs::read(&args.amiga_disk_filepath)?;
    let disk = Disk::try_create_with_data(disk_data)?;

    let fs: AmigaDos = disk.into();
    let data = fs.read(&args.amiga_input_filepath)?;

    if let Some(mut output) = get_output_file(args)? {
        output.write_all(&data)?;
    }

    Ok(())
}
