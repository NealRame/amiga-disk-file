use std::cell::RefCell;
use std::fs;
// use std::io::{
//     Write,
//     stdout,
// };
use std::path::PathBuf;
use std::rc::Rc;
use std::time::SystemTime;

use anyhow::Result;
use chrono::DateTime;

use nr_adf_lib::prelude::*;

fn parse_time_value(args: &Args) -> Result<SystemTime> {
    if let Some(s) = &args.date_time {
        let dt = DateTime::parse_from_rfc3339(s)?;
        Ok(dt.into())
    } else {
        Ok(SystemTime::now())
    }
}

/******************************************************************************
 * Format command run
 *****************************************************************************/
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Path to an Amiga disk file
    amiga_disk_filepath: PathBuf,

    /// Path to a file into the Amiga filesystem
    amiga_input_filepath: PathBuf,

    /// If destination file already exists, force overwriting it
    #[arg(short = 'c', long)]
    no_create: bool,

    /// Use specified time instead of current time
    #[arg(short = 't', long, value_name="RFC_3339_DATE_TIME")]
    date_time: Option<String>,
}

pub fn run(args: &Args) -> Result<()> {
    let disk_data = fs::read(&args.amiga_disk_filepath)?;
    let disk = Disk::try_create_with_data(disk_data)?;

    let fs = AmigaDos::try_from(Rc::new(RefCell::new(disk)))?;

    let time = parse_time_value(args)?;

    if args.no_create && !fs::exists(&args.amiga_input_filepath)? {
        return Ok(())
    }

    File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&fs, &args.amiga_input_filepath)?
        .set_time(&time)?;

    fs.dump(&args.amiga_disk_filepath)?;

    Ok(())
}
