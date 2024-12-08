use core::str;
use std::time::{
    UNIX_EPOCH,
    Duration,
    SystemTime,
};

use crate::disk::*;
use crate::errors::*;

use super::block::*;
use super::block_type::*;
use super::constants::*;


fn date_triplet_to_system_time(days: u32, mins: u32, ticks: u32) -> SystemTime {
    let seconds = ((days*24*60 + mins)*60 + ticks/TICKS_PER_SECOND) as u64;
    UNIX_EPOCH + AMIGA_EPOCH_OFFSET + Duration::from_secs(seconds)
}


#[derive(Clone, Debug)]
pub struct RootBlock {
    pub volume_name: String,

    pub root_creation_date: SystemTime,
    pub root_alteration_date: SystemTime,
    pub volume_alteration_date: SystemTime,

    block_primary_type: BlockPrimaryType,
    block_secondary_type: BlockSecondaryType,

    hash_table_size: u32,
    hash_table: [u32; ROOT_BLOCK_HASH_TABLE_MAX_SIZE],

    bitmap_flag: u32,
    bitmap_pages: [u32; ROOT_BLOCK_BITMAP_MAX_PAGES],
    bitmap_ext: u32,

    extension: u32,
}

impl Default for RootBlock {
    fn default() -> Self {
        let current_date = SystemTime::now();

        return Self {
            block_primary_type: BlockPrimaryType::Header,
            block_secondary_type: BlockSecondaryType::RootDir,

            hash_table_size: 0,
            hash_table: [0u32; ROOT_BLOCK_HASH_TABLE_MAX_SIZE],

            bitmap_flag: 0,
            bitmap_pages: [0u32; ROOT_BLOCK_BITMAP_MAX_PAGES],
            bitmap_ext: 0,

            volume_name: String::from("VOLUME"),

            root_creation_date: current_date,
            root_alteration_date: current_date,
            volume_alteration_date: current_date,

            extension: 0,
        };
    }
}

impl RootBlock {
    fn try_read_bitmap(
        &mut self,
        br: &BlockReader,
    ) -> Result<(), Error> {
        br.read_u32(ROOT_BLOCK_BITMAP_FLAG_OFFSET, &mut self.bitmap_flag)?;
        br.read_u32_array(ROOT_BLOCK_BITMAP_PAGES_OFFSET, &mut self.bitmap_pages)?;
        br.read_u32(ROOT_BLOCK_BITMAP_EXTENSION_OFFSET, &mut self.bitmap_ext)?;
        Ok(())
    }

    fn try_read_hash_table(
        &mut self,
        br: &BlockReader,
    ) -> Result<(), Error> {
        br.read_u32(ROOT_BLOCK_HASH_TABLE_SIZE_OFFSET, &mut self.hash_table_size)?;
        br.read_u32_array(ROOT_BLOCK_HASH_TABLE_OFFSET, &mut self.hash_table)?;
        Ok(())
    }

    fn try_read_root_alteration_date(
        &mut self,
        br: &BlockReader,
    ) -> Result<(), Error> {
        let mut days = 0u32;
        let mut mins = 0u32;
        let mut ticks = 0u32;

        br.read_u32(ROOT_BLOCK_R_DAYS_OFFSET, &mut days)?;
        br.read_u32(ROOT_BLOCK_R_MINS_OFFSET, &mut mins)?;
        br.read_u32(ROOT_BLOCK_R_TICKS_OFFSET, &mut ticks)?;

        self.root_alteration_date = date_triplet_to_system_time(days, mins, ticks);

        Ok(())
    }

    fn try_read_disk_alteration_date(
        &mut self,
        br: &BlockReader,
    ) -> Result<(), Error> {
        let mut days = 0u32;
        let mut mins = 0u32;
        let mut ticks = 0u32;

        br.read_u32(ROOT_BLOCK_V_DAYS_OFFSET, &mut days)?;
        br.read_u32(ROOT_BLOCK_V_MINS_OFFSET, &mut mins)?;
        br.read_u32(ROOT_BLOCK_V_TICKS_OFFSET, &mut ticks)?;

        self.volume_alteration_date = date_triplet_to_system_time(days, mins, ticks);

        Ok(())
    }

    fn try_read_root_creation_date(
        &mut self,
        br: &BlockReader,
    ) -> Result<(), Error> {
        let mut days = 0u32;
        let mut mins = 0u32;
        let mut ticks = 0u32;

        br.read_u32(ROOT_BLOCK_C_DAYS_OFFSET, &mut days)?;
        br.read_u32(ROOT_BLOCK_C_MINS_OFFSET, &mut mins)?;
        br.read_u32(ROOT_BLOCK_C_TICKS_OFFSET, &mut ticks)?;

        self.root_creation_date = date_triplet_to_system_time(days, mins, ticks);

        Ok(())
    }

    fn try_read_extension(
        &mut self,
        br: &BlockReader,
    ) -> Result<(), Error> {
        br.read_u32(ROOT_BLOCK_EXTENSION_OFFSET, &mut self.extension)?;
        Ok(())
    }

    fn try_read_volume_name(
        &mut self,
        br: &BlockReader,
    ) -> Result<(), Error> {
        let mut name =  [0; ROOT_BLOCK_DISK_NAME_MAX_SIZE];
        let mut name_size = 0;

        br.read_u8(
            ROOT_BLOCK_VOLUME_NAME_SIZE_OFFSET,
            &mut name_size,
        )?;
        br.read_u8_array(
            ROOT_BLOCK_VOLUME_NAME_OFFSET,
            &mut name,
        )?;

        if let Ok(name) = str::from_utf8(&name[..name_size as usize]) {
            self.volume_name = name.into();
        } else {
            return Err(Error::CorruptedImageFile);
        }

        Ok(())
    }

    pub fn try_read_from_disk(
        &mut self,
        disk: &Disk,
    ) -> Result<(), Error> {
        let addr = disk.block_count()/2;
        let reader = BlockReader::try_from_disk(disk, addr)?;

        reader.read_block_primary_type(&mut self.block_primary_type)?;
        reader.read_block_secondary_type(&mut self.block_secondary_type)?;

        self.try_read_bitmap(&reader)?;
        self.try_read_hash_table(&reader)?;

        self.try_read_volume_name(&reader)?;

        self.try_read_root_alteration_date(&reader)?;
        self.try_read_disk_alteration_date(&reader)?;
        self.try_read_root_creation_date(&reader)?;

        self.try_read_extension(&reader)?;

        Ok(())
    }
}
