use std::path::Path;

use crate::errors::*;

use super::amiga_dos::*;
use super::dir::*;
use super::file::*;
use super::path::*;


impl AmigaDos {
    /// Removes an empty directory.
    pub fn remove_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), Error> {
        let mut file = File::try_open(
            self,
            path.as_ref(),
            0|FileMode::Write,
        )?;

        file.set_len(0)?;

        let name = get_basename(path.as_ref())?;
        let parent_path = get_dirname(path.as_ref())?;
        let mut dir = Dir::try_with_path(
            self,
            parent_path
        )?;

        dir.remove_entry(name)?;

        self.inner.borrow_mut().free_block(file.header_block_address)?;

        Ok(())
    }
}
