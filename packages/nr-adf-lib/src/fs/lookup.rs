use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use crate::block::*;
use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::boot_block::*;
use super::constants::*;
use super::name::*;


fn path_split<P: AsRef<Path>>(
    path: P,
) -> Option<Vec<String>> {
    path.as_ref().to_str()
        .map(|path| path.split("/"))
        .map(|strs| strs.filter_map(|s| {
            if s.len() > 0 {
                Some(String::from(s))
            } else {
                None
            }
        }))
        .map(|res| res.collect::<Vec<String>>())
}

pub(super) fn lookup_entry(
    disk: Rc<RefCell<Disk>>,
    block_addr: LBAAddress,
    name: &str,
) -> Result<Option<LBAAddress>, Error> {
    let boot_block = BootBlockReader::try_from_disk(disk.clone())?;
    let international_mode = boot_block.get_international_mode();

    let hash_table = Block::new(disk.clone(), block_addr).read_hash_table()?;
    let hash_index = hash_name(&name, international_mode);
    let mut addr = hash_table[hash_index] as LBAAddress;

    while addr != 0 {
        let block = Block::new(disk.clone(), addr);
        let entry_name = block.read_name()?;

        if entry_name == name {
            return Ok(Some(addr));
        }

        addr = block.read_u32(BLOCK_HASH_CHAIN_NEXT_OFFSET)? as LBAAddress;
    }

    Ok(None)
}

impl AmigaDosInner {
    pub(super) fn lookup<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<LBAAddress, Error> {
        if let Some(path) = path_split(path) {
            let boot_block = BootBlockReader::try_from_disk(self.disk())?;
            let mut block_addr = boot_block.get_root_block_address();

            for name in path {
                if let Some(addr) = lookup_entry(self.disk(), block_addr, &name)? {
                    block_addr = addr;
                } else {
                    return Err(Error::NotFoundError);
                }
            }

            Ok(block_addr)
        } else {
            Err(Error::InvalidPathError)
        }
    }
}
