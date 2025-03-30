use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
// use std::time::SystemTime;

use crate::block::*;
use crate::disk::*;
use crate::errors::*;
use crate::fs::name::get_basename;
use crate::fs::name::get_dirname;

use super::amiga_dos::*;
// use super::block_type::*;
// use super::boot_block::*;
use super::constants::*;
use super::dir::*;
use super::metadata::*;
// use super::name::*;

pub(super) fn check_empty_directory(
    disk: Rc<RefCell<Disk>>,
    dir: &Dir,
) -> Result<bool, Error> {
    let block = Block::new(disk.clone(), dir.header_block_address);

    for index in 0..BLOCK_TABLE_SIZE {
        if block.read_block_table_address(index)?.is_some() {
            return Ok(false)
        }
    }

    Ok(true)
}

impl AmigaDos {
    /// Removes an empty directory.
    pub fn remove_dir<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), Error> {
        let dir = Dir::try_with_path(self, path.as_ref())?;

        check_empty_directory(self.disk(), &dir)?;

        let name = get_basename(path.as_ref())?;
        let parent_path = get_dirname(path.as_ref())?;
        let mut parent_dir = Dir::try_with_path(self, parent_path)?;

        parent_dir.remove_entry(name)?;

        self.inner.borrow_mut().free_block(dir.header_block_address)
    }

    // Removes a directory at this path, after removing all its contents.
    // Use carefully!
    // pub fn remove_dir_all<P: AsRef<Path>>(
    //     &self,
    //     path: P
    // ) -> Result<(), Error> {
    //     let entries =
    //         self.read_dir(path.as_ref())?
    //             .collect::<Result<Vec<_>, _>>()?;

    //     for entry in entries {
    //         let entry_path = entry.path();

    //         match entry.file_type() {
    //             FileType::Dir => {
    //                 self.remove_dir_all(entry_path)?
    //             },
    //             FileType::File => {
    //                 self.remove_file(entry_path)?;
    //             },
    //             _ => {
    //                 return Err(Error::BadFileDescriptor);
    //             }
    //         }
    //     }

    //     self.remove_dir(path)
    // }

}
