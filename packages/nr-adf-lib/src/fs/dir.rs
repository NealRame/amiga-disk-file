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
            block.write_block_chain_next_address(head.unwrap_or(0))?;
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
        let mut entry_block_addr_list = vec![];
        let hash_table = Block::new(
            disk.clone(),
            self.header_block_address,
        ).read_hash_table()?;

        for mut block_addr in hash_table.iter().copied().filter_map(AmigaDos::to_address) {
            while block_addr != 0 {
                entry_block_addr_list.push(block_addr);
                block_addr = Block::new(
                    disk.clone(),
                    block_addr,
                ).read_u32(BLOCK_HASH_CHAIN_NEXT_OFFSET)? as LBAAddress;
            }
        }

        Ok(DirIterator {
            disk: disk.clone(),
            entry_block_addr_list,
            path: self.path.clone(),
        })
    }
}
