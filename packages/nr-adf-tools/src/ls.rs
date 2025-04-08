use std::cell::RefCell;
use std::fs;
use std::path::{
    Path,
    PathBuf,
};
use std::rc::Rc;

use anyhow::{anyhow, Result};

use nr_adf_lib::prelude::*;

/******************************************************************************
 * List command options
 *****************************************************************************/
#[derive(clap::Args)]
pub struct Args {
    /// Path to an Amiga disk file
    amiga_disk_filepath: PathBuf,

    /// Path to a file into the Amiga filesystem
    amiga_input_filepath: PathBuf,

    /// Recursively list subdirectories encountered
    #[arg(short = 'r', long = "recurse")]
    recursive: bool
}

/******************************************************************************
 * List command run
 *****************************************************************************/

fn list_directory(args: &Args, fs: &AmigaDos, path: &Path) -> Result<()> {
    let mut dirs = Vec::<PathBuf>::new();

    println!("{}:", path.to_str().unwrap_or_default());

    for entry in fs.read_dir(path)? {
        match entry {
            Ok(entry) => {
                if FileType::Dir == entry.file_type() {
                    dirs.push(entry.path().into());
                }
                list_file(&entry.metadata())?;
            },
            Err(err) => {
                return Err(anyhow!("{}", err));
            }
        }
    }

    if args.recursive {
        for dirpath in dirs.iter() {
            println!();
            list_directory(args, fs, dirpath)?;
        }
    }

    Ok(())
}

fn list_file(metadata: &Metadata) -> Result<()> {
    println!("{}", metadata);
    Ok(())
}

pub fn run(args: &Args) -> Result<()> {
    let disk_data = fs::read(&args.amiga_disk_filepath)?;
    let disk = Disk::try_create_with_data(disk_data)?;
    let fs = AmigaDos::try_from(Rc::new(RefCell::new(disk)))?;

    let metadata = fs.metadata(&args.amiga_input_filepath)?;

    match metadata.file_type() {
        FileType::File => {
            list_file(&metadata)?;
        },
        FileType::Dir => {
            list_directory(args, &fs, &args.amiga_input_filepath)?;
        }
        _ => {},
    }

    // for entry in fs.read_dir(&args.amiga_input_filepath)? {
    //     match entry {
    //         Ok(entry) => {
    //             println!("{}", entry.name());
    //         },
    //         Err(err) => {
    //             return Err(anyhow!("{}", err));
    //         }
    //     }
    // }

    Ok(())
}
