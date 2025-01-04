use core::str;

use std::time::SystemTime;

use crate::disk::*;
use crate::errors::*;

use super::block::*;
use super::block_type::*;
use super::constants::*;
use super::options::*;


#[derive(Clone, Debug)]
pub struct RootBlockWriter {
    filesystem_type: FilesystemType,
    volume_name: String,
    root_block_address: Option<LBAAddress>,
}

impl Default for RootBlockWriter {
    fn default() -> Self {
        Self {
            filesystem_type: FilesystemType::OFS,
            volume_name: String::from("VOLUME"),
            root_block_address: None,
        }
    }
}

impl RootBlockWriter {
    pub fn with_filesystem_type(
        &mut self,
        filesystem_type: FilesystemType,
    ) -> &mut Self {
        self.filesystem_type = filesystem_type;
        self
    }

    pub fn with_volume(
        &mut self,
        volume_name: &str,
    ) -> &mut Self {
        self.volume_name = volume_name.into();
        self
    }

    pub fn write(
        &self,
        disk: &mut Disk,
    ) -> Result<(), Error> {
        let datetime = SystemTime::now();

        let mut writer = BlockWriter::try_from_disk(
            disk,
            self.root_block_address.unwrap_or_else(|| disk.block_count()/2),
        )?;

        writer.clear();
        writer.write_block_primary_type(BlockPrimaryType::Header)?;
        writer.write_block_secondary_type(BlockSecondaryType::Root)?;

        writer.write_alteration_date(&datetime)?;
        writer.write_disk_alteration_date(&datetime)?;
        writer.write_root_creation_date(&datetime)?;

        writer.write_name(&self.volume_name)?;

        writer.write_u32(
            ROOT_BLOCK_HASH_TABLE_SIZE_OFFSET,
            BLOCK_HASH_TABLE_SIZE as u32
        )?;
        writer.write_u32(ROOT_BLOCK_BITMAP_FLAG_OFFSET, 0xffffffff)?;
        writer.write_u32(ROOT_BLOCK_EXTENSION_OFFSET, 0)?;

        writer.write_checksum(BLOCK_CHECKSUM_OFFSET)?;

        Ok(())
    }
}
