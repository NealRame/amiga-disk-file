use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use crate::block::*;
use crate::disk::*;
use crate::errors::*;
use crate::fs::path::*;

use super::amiga_dos::*;
use super::constants::*;
use super::dir::*;

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
}
