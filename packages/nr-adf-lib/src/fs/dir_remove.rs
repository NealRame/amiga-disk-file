// use std::cell::RefCell;
use std::path::Path;
// use std::rc::Rc;
// use std::time::SystemTime;

// use crate::block::*;
// use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
// use super::block_type::*;
// use super::boot_block::*;
// use super::constants::*;
// use super::dir::*;
// use super::metadata::*;
// use super::name::*;

impl AmigaDos {
    /// Removes an empty directory.
    pub fn remove_dir<P: AsRef<Path>>(
        &self,
        _: P,
    ) -> Result<(), Error> {
        // let addr = self.lookup(path)?;

        // let block = Block::new(self.disk(), addr);

        // if let Some(parent_block_addr) = block.read_parent_block_address()? {

        // }

        // Ok(())
        unimplemented!()
    }
}
