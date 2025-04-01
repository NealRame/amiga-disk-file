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
        let metadata = self.metadata(path.as_ref())?;

        if metadata.is_file() {
            let header_block_address = metadata.header_block_address();

            let mut file = File::try_open_with_block_address(
                self,
                metadata.header_block_address(),
                0|FileMode::Write
            )?;

            file.set_len(0)?;

            let name = get_basename(path.as_ref())?;
            let parent_path = get_dirname(path.as_ref())?;
            let mut dir = Dir::try_with_path(
                self,
                parent_path
            )?;

            dir.remove_entry(name)?;

            self.inner.borrow_mut().free_block(header_block_address)?;

            Ok(())
        } else {
            Err(Error::NotAFileError)
        }
    }
}
