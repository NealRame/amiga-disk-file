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
use super::name::*;


pub(super) fn check_directory(
    disk: Rc<RefCell<Disk>>,
    addr: LBAAddress,
) -> Result<(), Error> {
    let block = Block::new(disk.clone(), addr);

    match (
        block.read_block_primary_type()?,
        block.read_block_secondary_type()?,
    ) {
        (BlockPrimaryType::Header, BlockSecondaryType::Directory) |
        (BlockPrimaryType::Header, BlockSecondaryType::Root) => Ok(()),
        _ => {
            Err(Error::NotADirectoryError)
        }
    }
}

pub(super) fn check_empty_directory(
    disk: Rc<RefCell<Disk>>,
    addr: LBAAddress,
) -> Result<bool, Error> {
    check_directory(disk.clone(), addr)?;

    let block = Block::new(disk.clone(), addr);

    for index in 0..BLOCK_TABLE_SIZE {
        if block.read_block_table_address(index)?.is_some() {
            return Ok(false)
        }
    }

    Ok(true)
}

pub(super) fn find_in_hash_chain(
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


#[derive(Clone, Debug)]
pub(super) struct Dir {
    fs: Rc<RefCell<AmigaDosInner>>,
    pub(super) header_block_address: LBAAddress,
    pub(super) path: PathBuf,
}

impl Dir {
    pub(super) fn try_with_block_address<P: AsRef<Path>>(
        fs: &AmigaDos,
        header_block_address: LBAAddress,
        path: P,
    ) -> Result<Self, Error> {
        check_directory(fs.disk(), header_block_address)?;

        Ok(Self {
            fs: fs.inner.clone(),
            header_block_address,
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

impl Dir {
    pub(super) fn lookup(
        &self,
        name: &str,
    ) -> Result<Option<LBAAddress>, Error> {
        let disk = self.fs.borrow().disk();
        let boot_block = BootBlockReader::try_from_disk(disk.clone())?;
        let international_mode = boot_block.get_international_mode();

        let hash_index = hash_name(name, international_mode);
        let head = Block::new(
            disk.clone(),
            self.header_block_address,
        ).read_block_table_address(hash_index)?;

        find_in_hash_chain(disk.clone(), name, head)
    }

    pub(super) fn add_entry(
        &mut self,
        name: &str,
        entry_block_address: LBAAddress,
    ) -> Result<(), Error> {
        let disk = self.fs.borrow().disk();

        let boot_block = BootBlockReader::try_from_disk(disk.clone())?;
        let international_mode = boot_block.get_international_mode();

        let hash_index = hash_name(name, international_mode);
        let hash_chain_head = Block::new(
            disk.clone(),
            self.header_block_address,
        ).read_block_table_address(hash_index)?;

        if find_in_hash_chain(disk.clone(), name, hash_chain_head)?.is_some() {
            panic!("guru meditation: {} already exists!", name);
        }

        let mut dir_block = Block::new(
            self.fs.borrow().disk(),
            self.header_block_address,
        );

        dir_block.write_alteration_date(&SystemTime::now())?;
        dir_block.write_hash_table_block_address(
            hash_index,
            entry_block_address,
        )?;
        dir_block.write_checksum()?;

        let mut entry_block = Block::new(
            disk.clone(),
            entry_block_address,
        );

        entry_block.write_hash_chain_next_address(hash_chain_head.unwrap_or(0))?;
        entry_block.write_u32(
            BLOCK_PARENT_OFFSET,
            entry_block_address as u32,
        )?;
        entry_block.write_checksum()?;

        Ok(())
    }

    pub(super) fn remove_entry(
        &mut self,
        name: &str,
    ) -> Result<(), Error> {
        let disk = self.fs.borrow().disk();

        let boot_block = BootBlockReader::try_from_disk(disk.clone())?;
        let international_mode = boot_block.get_international_mode();

        let hash_index = hash_name(name, international_mode);

        let mut prev_addr = None;
        let mut next_addr = None;

        let mut curr_addr = Block::new(
            disk.clone(),
            self.header_block_address,
        ).read_block_table_address(hash_index)?;

        while curr_addr.is_some() {
            let curr_block = Block::new(disk.clone(), curr_addr.unwrap());
            let curr_name = curr_block.read_name()?;

            next_addr = AmigaDos::to_address(curr_block.read_u32(BLOCK_HASH_CHAIN_NEXT_OFFSET)?);

            if name == curr_name {
                break;
            }

            prev_addr = curr_addr;
            curr_addr = next_addr;
        }

        let mut dir_block = Block::new(
            self.fs.borrow().disk(),
            self.header_block_address,
        );

        if prev_addr.is_some() {
            Block::new(
                disk.clone(),
                prev_addr.unwrap(),
            ).write_hash_chain_next_address(next_addr.unwrap_or(0))?;
        } else {
            dir_block.write_hash_table_block_address(hash_index, 0)?;
        }

        dir_block.write_alteration_date(&SystemTime::now())?;
        dir_block.write_checksum()?;

        Ok(())
    }
}
