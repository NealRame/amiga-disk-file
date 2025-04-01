use std::path::Path;
use std::path::PathBuf;

use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::boot_block::*;
use super::dir::*;
use super::path::*;


impl AmigaDos {
    pub(super) fn lookup<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<LBAAddress, Error> {
        if let Some(path) = split(path) {
            let disk = self.inner.borrow().disk();

            let boot_block = BootBlockReader::try_from_disk(disk.clone())?;

            let mut current_block_addr = boot_block.get_root_block_address();
            let mut current_path = PathBuf::from("/");

            for name in path {
                let dir = Dir::try_with_block_address(
                    self,
                    current_block_addr,
                    &current_path,
                )?;

                if let Some(addr) = dir.lookup(&name)? {
                    current_block_addr = addr;
                    current_path = current_path.join(name);
                } else {
                    return Err(Error::NotFoundError);
                }
            }

            Ok(current_block_addr)
        } else {
            Err(Error::InvalidPathError)
        }
    }
}
