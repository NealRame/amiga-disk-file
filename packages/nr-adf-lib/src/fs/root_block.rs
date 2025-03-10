use core::str;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::SystemTime;

use crate::block::*;
use crate::disk::*;
use crate::errors::*;

use super::amiga_dos_options::*;
use super::block_type::*;
use super::constants::*;


#[derive(Clone, Debug)]
pub struct RootBlockInitializer {
    filesystem_type: FilesystemType,
    volume_name: String,
    root_block_address: Option<LBAAddress>,
}

impl Default for RootBlockInitializer {
    fn default() -> Self {
        Self {
            filesystem_type: FilesystemType::OFS,
            volume_name: String::from("VOLUME"),
            root_block_address: None,
        }
    }
}

impl RootBlockInitializer {
    pub fn with_filesystem_type(
        &mut self,
        filesystem_type: FilesystemType,
    ) -> &mut Self {
        self.filesystem_type = filesystem_type;
        self
    }

    pub fn with_root_block_address(
        &mut self,
        addr: Option<LBAAddress>,
    ) -> &mut Self {
        self.root_block_address = addr;
        self
    }

    pub fn with_volume_name(
        &mut self,
        volume_name: &str,
    ) -> &mut Self {
        self.volume_name = volume_name.into();
        self
    }

    pub fn init(
        &self,
        disk: Rc<RefCell<Disk>>,
    ) -> Result<(), Error> {

        let datetime = SystemTime::now();
        let root_block_addr = self.root_block_address.unwrap_or_else(|| {
            disk.borrow().block_count()/2
        });
        let mut block = Block::new(disk, root_block_addr);

        block.clear()?;
        block.write_block_primary_type(BlockPrimaryType::Header)?;
        block.write_block_secondary_type(BlockSecondaryType::Root)?;

        block.write_alteration_date(&datetime)?;
        block.write_disk_alteration_date(&datetime)?;
        block.write_root_creation_date(&datetime)?;

        block.write_name(&self.volume_name)?;

        block.write_u32(
            ROOT_BLOCK_HASH_TABLE_SIZE_OFFSET,
            BLOCK_TABLE_SIZE as u32,
        )?;
        block.write_u32(ROOT_BLOCK_BITMAP_FLAG_OFFSET, 0xffffffff)?;
        block.write_u32(ROOT_BLOCK_EXTENSION_OFFSET, 0)?;

        block.write_checksum(BLOCK_CHECKSUM_OFFSET)?;

        Ok(())
    }
}
