use std::cell::RefCell;
use std::path::{
    Path,
    PathBuf,
};
use std::rc::Rc;

use crate::block::*;
use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::constants::*;
use super::dir::*;
use super::metadata::*;


#[derive(Clone, Debug)]
pub struct DirEntry {
    metadata: Metadata,
    name: String,
    path: PathBuf,
}

impl DirEntry {
    pub fn file_type(&self) -> FileType {
        self.metadata.file_type()
    }

    pub fn metadata(&self) -> Metadata {
        self.metadata.clone()
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct DirIterator {
    current_table_index: usize,
    current_table_addr: Option<LBAAddress>,
    disk: Rc<RefCell<Disk>>,
    header_block_address: LBAAddress,
    path: PathBuf,
}

impl DirIterator {
    fn block_table_next(
        &mut self
    ) -> Result<(), Error> {
        let disk = self.disk.clone();
        let header_block = Block::new(disk, self.header_block_address);

        while self.current_table_addr.is_none()
            && self.current_table_index < BLOCK_TABLE_SIZE {
            self.current_table_addr = header_block.read_block_table_address(self.current_table_index)?;
            self.current_table_index += 1;
        }

        Ok(())
    }

    fn block_chain_next(
        &mut self,
        block: &Block,
    ) -> Result<(), Error> {
        self.current_table_addr = match block.read_hash_chain_next_address() {
            Ok(addr) => addr,
            Err(err) => {
                return Err(err);
            }
        };
        Ok(())
    }
}

impl Iterator for DirIterator {
    type Item = Result<DirEntry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Err(err) = self.block_table_next() {
            return Some(Err(err));
        }

        let block_addr = self.current_table_addr?;
        let block = Block::new(self.disk.clone(), block_addr);

        let metadata = match Metadata::try_from(&block) {
            Ok(metadata) => metadata,
            Err(err) => {
                return Some(Err(err));
            }
        };

        let name = match block.read_name() {
            Ok(name) => name,
            Err(err) => {
                return Some(Err(err));
            }
        };

        let path = self.path.join(&name);

        if let Err(err) = self.block_chain_next(&block) {
            return Some(Err(err));
        }

        Some(Ok(DirEntry { metadata, name, path }))
    }
}

impl AmigaDos {
    /// Returns an iterator over the entries within a directory.
    pub fn read_dir<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<DirIterator, Error> {
        let dir = Dir::try_with_path(self, path)?;

        Ok(DirIterator {
            current_table_index: 0,
            current_table_addr: None,
            disk: self.disk(),
            header_block_address: dir.header_block_address,
            path: dir.path.clone(),
        })
    }
}
