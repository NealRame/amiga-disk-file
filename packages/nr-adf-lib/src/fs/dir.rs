use std::cell::RefCell;
use std::path::{
    Path,
    PathBuf,
};
use std::rc::Rc;
use std::time::SystemTime;

use crate::block::*;
use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::block_type::*;
use super::boot_block::*;
use super::constants::*;
use super::metadata::*;
use super::name::*;


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

#[derive(Clone, Debug)]
pub(super) struct Dir {
    fs: Rc<RefCell<AmigaDosInner>>,
    pub(super) header_block_address: LBAAddress,
    pub(super) path: PathBuf,
}

impl Dir {
    pub(super) fn try_with_block_address<P: AsRef<Path>>(
        fs: &AmigaDos,
        block_addr: LBAAddress,
        path: P,
    ) -> Result<Self, Error> {
        Ok(Self {
            fs: fs.inner.clone(),
            header_block_address: block_addr,
            path: PathBuf::from(path.as_ref()),
        })
    }

    pub(super) fn try_with_path<P: AsRef<Path>>(
        fs: &AmigaDos,
        path: P,
    ) -> Result<Self, Error> {
        let block_addr = fs.lookup(path.as_ref())?;

        Self::try_with_block_address(
            fs,
            block_addr,
            path.as_ref(),
        )
    }
}

fn find_entry_in_hash_chain(
    disk: Rc<RefCell<Disk>>,
    name: &str,
    mut addr: Option<LBAAddress>,
) -> Result<Option<LBAAddress>, Error> {
    while let Some(block_addr) = addr {
        let block = Block::new(disk.clone(), block_addr);
        let entry_name = block.read_name()?;

        if entry_name == name {
            return Ok(Some(block_addr));
        }
        addr = AmigaDos::to_address(block.read_u32(BLOCK_HASH_CHAIN_NEXT_OFFSET)?);
    }

    Ok(None)
}

impl Dir {
    pub(super) fn lookup(
        &self,
        name: &str,
    ) -> Result<Option<LBAAddress>, Error> {
        let disk = self.fs.borrow().disk();
        let boot_block = BootBlockReader::try_from_disk(disk.clone())?;
        let international_mode = boot_block.get_international_mode();

        let hash_index = hash_name(&name, international_mode);
        let head = Block::new(
            disk.clone(),
            self.header_block_address,
        ).read_block_table_address(hash_index)?;

        find_entry_in_hash_chain(disk.clone(), name, head)
    }

    pub(super) fn create_entry(
        &mut self,
        name: &str,
        file_type: FileType,
    ) -> Result<bool, Error> {
        let disk = self.fs.borrow().disk();

        let boot_block = BootBlockReader::try_from_disk(disk.clone())?;
        let international_mode = boot_block.get_international_mode();

        let hash_index = hash_name(&name, international_mode);
        let head = Block::new(
            disk.clone(),
            self.header_block_address,
        ).read_block_table_address(hash_index)?;

        if let None = find_entry_in_hash_chain(disk.clone(), name, head)? {
            Ok(false)
        } else {
            let header_block_address = self.fs.borrow_mut().reserve_block()?;
            let mut block = Block::new(disk.clone(), header_block_address);

            block.clear()?;
            block.write_block_primary_type(BlockPrimaryType::Header)?;
            block.write_block_secondary_type(if file_type == FileType::Dir {
                BlockSecondaryType::Directory
            } else {
                BlockSecondaryType::File
            })?;

            block.write_alteration_date(&SystemTime::now())?;
            block.write_hash_chain_next_address(head.unwrap_or(0))?;
            block.write_name(name)?;
            block.write_u32(
                BLOCK_DATA_LIST_HEADER_KEY_OFFSET,
                header_block_address as u32,
            )?;
            block.write_u32(
                BLOCK_DATA_LIST_PARENT_OFFSET,
                self.header_block_address as u32,
            )?;

            Block::new(
                disk.clone(),
                self.header_block_address,
            ).write_block_table_address(hash_index, header_block_address)?;

            Ok(true)
        }
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

        if self.current_table_addr.is_none() {
            return None;
        }

        let block = Block::new(
            self.disk.clone(),
            self.current_table_addr.unwrap(),
        );

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
