use std::path::{
    Path,
    PathBuf,
};
use std::time::SystemTime;

use crate::block::*;
use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::block_type::*;
use super::boot_block::*;
use super::constants::*;
use super::dir::*;
use super::name::*;
use super::path_split::*;


fn init_dir_block_header(
    fs: &AmigaDos,
    name: &str,
) -> Result<LBAAddress, Error> {
    let block_addr = fs.inner.borrow_mut().reserve_block()?;
    let mut block = Block::new(fs.disk(), block_addr);

    block.clear()?;

    block.write_block_primary_type(BlockPrimaryType::Header)?;
    block.write_block_secondary_type(BlockSecondaryType::Directory)?;
    block.write_alteration_date(&SystemTime::now())?;
    block.write_name(name)?;
    block.write_u32(
        BLOCK_DATA_LIST_HEADER_KEY_OFFSET,
        block.address as u32,
    )?;

    Ok(block_addr)
}

impl AmigaDos {
    /// Creates a new, empty directory at the provided path.
    /// Errors:
    /// - When a parent of the given path doesn’t exist. Use `create_dir_all`
    ///   function to create a directory and all its missing parents at the
    ///   same time.
    pub fn create_dir<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<(), Error> {
        let path = path.as_ref();

        let parent_path = get_dirname(path)?;
        let name = get_basename(path)?;

        let mut dir = Dir::try_with_path(self, parent_path)?;

        if let Some(addr) = dir.lookup(name)? {
            check_directory(self.disk(), addr)
        } else {
            let addr = init_dir_block_header(self, name)?;

            dir.add_entry(name, addr)?;
            Ok(())
        }
    }

    /// Create a directory and all of its parent components if they are
    /// missing.
    pub fn create_dir_all<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<(), Error> {
        if let Some(path) = path_split(path) {
            let disk = self.inner.borrow().disk();

            let boot_block = BootBlockReader::try_from_disk(disk.clone())?;
            let mut dir = Dir::try_with_block_address(
                self,
                boot_block.get_root_block_address(),
                PathBuf::default(),
            )?;

            for name in path {
                let addr = if let Some(addr) = dir.lookup(&name)? {
                    check_directory(self.disk(), addr)?;
                    addr
                } else {
                    let addr = init_dir_block_header(self, &name)?;

                    dir.add_entry(&name, addr)?;
                    addr
                };

                dir = Dir::try_with_block_address(
                    self,
                    addr,
                    PathBuf::default(),
                )?;
            }

            Ok(())
        } else {
            Err(Error::InvalidPathError)
        }
    }
}
