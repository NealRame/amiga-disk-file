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
use super::lookup::*;
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
        self.metadata
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }
}

fn non_zero(addr: &u32) -> bool {
    *addr != 0
}


#[derive(Clone, Debug)]
pub(super) struct Dir {
    pub(super) fs: Rc<RefCell<AmigaDosInner>>,
    pub(super) header_block_address: LBAAddress,
    pub(super) path: PathBuf,
}

impl Dir {
    fn try_with_block_address<P: AsRef<Path>>(
        fs: Rc<RefCell<AmigaDosInner>>,
        block_addr: LBAAddress,
        path: P,
    ) -> Result<Self, Error> {
        Ok(Self {
            fs,
            header_block_address: block_addr,
            path: PathBuf::from(path.as_ref()),
        })
    }

    pub(super) fn try_with_path<P: AsRef<Path>>(
        fs: Rc<RefCell<AmigaDosInner>>,
        path: P,
    ) -> Result<Self, Error> {
        let block_addr = fs.borrow().lookup(path.as_ref())?;

        Self::try_with_block_address(
            fs,
            block_addr,
            path.as_ref(),
        )
    }
}

impl Dir {
    pub(super) fn lookup(
        &self,
        name: &str,
    ) -> Result<Option<LBAAddress>, Error> {
        lookup_entry(self.fs.borrow().disk(), self.header_block_address, name)
    }

    pub(super) fn create_file(
        &mut self,
        name: &str,
    ) -> Result<Option<LBAAddress>, Error> {
        unimplemented!()
    }

    pub(super) fn create_dir(
        &mut self,
        name: &str,
    ) -> Result<Option<LBAAddress>, Error> {
        unimplemented!()
    }
}

pub struct DirIterator {
    disk: Rc<RefCell<Disk>>,
    entry_block_addr_list: Vec<LBAAddress>,
    path: PathBuf,
}

impl Iterator for DirIterator {
    type Item = Result<DirEntry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(addr) = self.entry_block_addr_list.pop() {
            let block = Block::new(self.disk.clone(), addr);

            let metadata = match Metadata::try_from(&block) {
                Ok(metadata) => metadata,
                Err(err) => {
                    return Some(Err(err));
                },
            };

            let name = match block.read_name() {
                Ok(name) => name,
                Err(err) => {
                    return Some(Err(err));
                },
            };

            let path = self.path.join(&name);

            Some(Ok(DirEntry {
                metadata,
                name,
                path,
            }))
        } else {
            None
        }
    }
}

impl Dir {
    pub(super) fn read(
        &self,
    ) -> Result<DirIterator, Error> {
        let disk = self.fs.borrow().disk();

        let hash_table = Block::new(
            disk.clone(),
            self.header_block_address,
        ).read_hash_table()?;

        let mut entry_block_addr_list = vec![];
        for mut block_addr in hash_table.iter().copied().filter(non_zero) {
            while block_addr != 0 {
                entry_block_addr_list.push(block_addr as LBAAddress);
                block_addr = Block::new(
                    disk.clone(),
                    block_addr as LBAAddress,
                ).read_u32(BLOCK_HASH_CHAIN_NEXT_OFFSET)?;
            }
        }

        Ok(DirIterator {
            disk: disk.clone(),
            entry_block_addr_list,
            path: self.path.clone(),
        })
    }
}
