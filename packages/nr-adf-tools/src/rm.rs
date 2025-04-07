use std::cell::RefCell;
use std::fs;
use std::path::{
    Path,
    PathBuf,
};
use std::rc::Rc;

use anyhow::{anyhow, Result};

use promptly::prompt_default;

use nr_adf_lib::prelude::*;


/******************************************************************************
 * Remove command options
 *****************************************************************************/
#[derive(clap::Args)]
pub struct Args {
    /// Path to an Amiga disk file
    amiga_disk_filepath: PathBuf,

    /// Path to a file or a directory into the Amiga filesystem
    amiga_input_files: Vec<PathBuf>,

    /// Attempt to remove the files without prompting for confirmation
    #[arg(short = 'f', long)]
    force: bool,

    /// Attempt to remove the file hierarchy rooted in each file argument
    #[arg(short = 'r', long)]
    recursive: bool
}

/******************************************************************************
 * Format command run
 *****************************************************************************/
fn confirm_examine(
    args: &Args,
    dirpath: &Path,
) -> Result<bool> {
    if args.force {
        Ok(true)
    } else {
        prompt_default(
            format!("examine files in directory '{}'?", dirpath.to_str().unwrap()),
            false,
        ).map_err(|err| anyhow!(err.to_string()))
    }
}

fn confirm_remove(
    args: &Args,
    filepath: &Path,
) -> Result<bool> {
    if args.force {
        Ok(true)
    } else {
        prompt_default(
            format!("remove '{}'?", filepath.to_str().unwrap()),
            false,
        ).map_err(|err| anyhow!(err.to_string()))
    }
}

fn remove_dir(
    args: &Args,
    fs: &mut AmigaDos,
    path: &Path,
) -> Result<()> {
    if confirm_examine(args, path)? {
        let entries = fs.read_dir(path)?.collect::<Result<Vec<_>, _>>()?;

        for entry in entries {
            let entry_path = entry.path();

            match entry.file_type() {
                FileType::Dir => {
                    remove_dir(args, fs, entry_path)?;
                },
                FileType::File => {
                    remove_file(args, fs, entry_path)?;
                },
                _ => {
                    return Err(anyhow!("Unsupported file type!"));
                }
            }
        }

        if confirm_remove(args, path)? {
            fs.remove_dir(path)?;
        }
    }
    Ok(())
}

fn remove_file(
    args: &Args,
    fs: &mut AmigaDos,
    path: &Path,
) -> Result<()> {
    if confirm_remove(args, path)? {
        fs.remove_file(path)?;
    }
    Ok(())
}

pub fn run(args: &Args) -> Result<()> {
    let disk_data = fs::read(&args.amiga_disk_filepath)?;
    let disk = Disk::try_create_with_data(disk_data)?;
    let mut fs = AmigaDos::try_from(Rc::new(RefCell::new(disk)))?;

    for input_filepath in args.amiga_input_files.iter() {
        let metadata = fs.metadata(input_filepath)?;

        match metadata.file_type() {
            FileType::Dir if args.recursive => {
                remove_dir(args, &mut fs, input_filepath)?;
            },
            FileType::File => {
                remove_file(args, &mut fs, input_filepath)?;
            },
            FileType::Dir => {
                return Err(anyhow!(
                    "'{}' is a directory",
                    input_filepath.to_str().ok_or(Error::InvalidPathError)?,
                ));
            },
            FileType::Link => {
                return Err(anyhow!(
                    "'{}' is a link",
                    input_filepath.to_str().ok_or(Error::InvalidPathError)?,
                ));
            }
        }
    }

    fs.dump(&args.amiga_disk_filepath)?;

    Ok(())
}
