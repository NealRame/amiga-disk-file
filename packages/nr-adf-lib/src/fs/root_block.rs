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


fn date_triplet_from_system_time(date_time: &SystemTime) -> (u32, u32, u32) {
    const SECONDS_PER_DAYS: u32 = 24*60*60;
    const SECONDS_PER_MINS: u32 = 60;

    match date_time.duration_since(UNIX_EPOCH + AMIGA_EPOCH_OFFSET) {
        Ok(duration) => {
            let seconds = duration.as_secs() as u32;
            let (days, seconds) = (
                seconds/SECONDS_PER_DAYS,
                seconds%SECONDS_PER_DAYS,
            );
            let (mins, seconds) = (
                seconds/SECONDS_PER_MINS,
                seconds%SECONDS_PER_MINS,
            );

            (days, mins, seconds*TICKS_PER_SECOND)
        },
        _ => {
            (0, 0, 0)
        }
    }
}

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
    pub fn with_volume_name(volume_name: &str) -> Self {
        let mut root_block = RootBlock::default();

        root_block.volume_name = volume_name.into();
        root_block
    }
}

impl RootBlock {
    fn try_read_bitmap(
        &mut self,
        br: &BlockReader,
    ) -> Result<(), Error> {
        self.bitmap_flag = br.read_u32(ROOT_BLOCK_BITMAP_FLAG_OFFSET)?;
        self.bitmap_ext = br.read_u32(ROOT_BLOCK_BITMAP_EXTENSION_OFFSET)?;
        br.read_u32_array(ROOT_BLOCK_BITMAP_PAGES_OFFSET, &mut self.bitmap_pages)?;
        Ok(())
    }

    fn try_read_hash_table(
        &mut self,
        br: &BlockReader,
    ) -> Result<(), Error> {
        self.hash_table_size = br.read_u32(ROOT_BLOCK_HASH_TABLE_SIZE_OFFSET)?;
        br.read_u32_array(ROOT_BLOCK_HASH_TABLE_OFFSET, &mut self.hash_table)?;
        Ok(())
    }

    fn try_read_disk_alteration_date(
        &mut self,
        br: &BlockReader,
    ) -> Result<(), Error> {
        let days = br.read_u32(ROOT_BLOCK_V_DAYS_OFFSET)?;
        let mins = br.read_u32(ROOT_BLOCK_V_MINS_OFFSET)?;
        let ticks = br.read_u32(ROOT_BLOCK_V_TICKS_OFFSET)?;

        self.volume_alteration_date = date_triplet_to_system_time(days, mins, ticks);
        Ok(())
    }

    fn try_read_root_alteration_date(
        &mut self,
        br: &BlockReader,
    ) -> Result<(), Error> {
        let days = br.read_u32(ROOT_BLOCK_R_DAYS_OFFSET)?;
        let mins = br.read_u32(ROOT_BLOCK_R_MINS_OFFSET)?;
        let ticks = br.read_u32(ROOT_BLOCK_R_TICKS_OFFSET)?;

        self.root_alteration_date = date_triplet_to_system_time(days, mins, ticks);
        Ok(())
    }

    fn try_read_root_creation_date(
        &mut self,
        br: &BlockReader,
    ) -> Result<(), Error> {
        let days = br.read_u32(ROOT_BLOCK_C_DAYS_OFFSET)?;
        let mins = br.read_u32(ROOT_BLOCK_C_MINS_OFFSET)?;
        let ticks = br.read_u32(ROOT_BLOCK_C_TICKS_OFFSET)?;

        self.root_creation_date = date_triplet_to_system_time(days, mins, ticks);
        Ok(())
    }

    fn try_read_volume_name(
        &mut self,
        br: &BlockReader,
    ) -> Result<(), Error> {
        let mut name = [0u8; ROOT_BLOCK_DISK_NAME_MAX_SIZE];
        let name_size = br.read_u8(ROOT_BLOCK_VOLUME_NAME_SIZE_OFFSET)?;

        br.read_u8_array(ROOT_BLOCK_VOLUME_NAME_OFFSET, &mut name)?;

        if let Ok(name) = str::from_utf8(&name[..name_size as usize]) {
            self.volume_name = name.into();
        } else {
            return Err(Error::CorruptedImageFile);
        }

        Ok(())
    }

    fn try_read_extension(
        &mut self,
        br: &BlockReader,
    ) -> Result<(), Error> {
        self.extension = br.read_u32(ROOT_BLOCK_EXTENSION_OFFSET)?;
        Ok(())
    }

    pub fn try_read_from_disk(
        &mut self,
        disk: &Disk,
    ) -> Result<(), Error> {
        let addr = disk.block_count()/2;
        let reader = BlockReader::try_from_disk(disk, addr)?;

        reader.verify_checksum()?;
        reader.verify_block_primary_type(BlockPrimaryType::Header)?;
        reader.verify_block_secondary_type(BlockSecondaryType::RootDir)?;

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

impl RootBlock {
    fn try_write_bitmap(
        &self,
        bw: &mut BlockWriter,
    ) -> Result<(), Error> {
        bw.write_u32(ROOT_BLOCK_BITMAP_FLAG_OFFSET, self.bitmap_flag)?;
        bw.write_u32_array(ROOT_BLOCK_BITMAP_PAGES_OFFSET, &self.bitmap_pages)?;
        Ok(())

    }

    fn try_write_hash_table(
        &self,
        bw: &mut BlockWriter,
    ) -> Result<(), Error> {
        bw.write_u32(ROOT_BLOCK_HASH_TABLE_SIZE_OFFSET, self.hash_table_size)?;
        bw.write_u32_array(ROOT_BLOCK_HASH_TABLE_OFFSET, &self.hash_table)?;
        Ok(())
    }

    fn try_write_disk_alteration_date(
        &self,
        bw: &mut BlockWriter,
    ) -> Result<(), Error> {
        let (
            days,
            mins,
            ticks,
        ) = date_triplet_from_system_time(&self.root_alteration_date);

        bw.write_u32(ROOT_BLOCK_V_DAYS_OFFSET, days)?;
        bw.write_u32(ROOT_BLOCK_V_MINS_OFFSET, mins)?;
        bw.write_u32(ROOT_BLOCK_V_TICKS_OFFSET, ticks)?;

        Ok(())
    }

    fn try_write_root_alteration_date(
        &self,
        bw: &mut BlockWriter,
    ) -> Result<(), Error> {
        let (
            days,
            mins,
            ticks,
        ) = date_triplet_from_system_time(&self.root_alteration_date);

        bw.write_u32(ROOT_BLOCK_R_DAYS_OFFSET, days)?;
        bw.write_u32(ROOT_BLOCK_R_MINS_OFFSET, mins)?;
        bw.write_u32(ROOT_BLOCK_R_TICKS_OFFSET, ticks)?;

        Ok(())
    }

    fn try_write_root_creation_date(
        &self,
        bw: &mut BlockWriter,
    ) -> Result<(), Error> {
        let (
            days,
            mins,
            ticks,
        ) = date_triplet_from_system_time(&self.root_creation_date);

        bw.write_u32(ROOT_BLOCK_C_DAYS_OFFSET, days)?;
        bw.write_u32(ROOT_BLOCK_C_MINS_OFFSET, mins)?;
        bw.write_u32(ROOT_BLOCK_C_TICKS_OFFSET, ticks)?;

        Ok(())
    }

    fn try_write_volume_name(
        &self,
        bw: &mut BlockWriter,
    ) -> Result<(), Error> {
        let name = self.volume_name.as_bytes();
        let name_size = usize::min(name.len(), ROOT_BLOCK_DISK_NAME_MAX_SIZE);

        bw.write_u8(ROOT_BLOCK_VOLUME_NAME_SIZE_OFFSET, name_size as u8)?;
        bw.write_u8_array(ROOT_BLOCK_VOLUME_NAME_OFFSET, &name[..name_size])?;

        Ok(())
    }

    fn try_write_extension(
        &self,
        bw: &mut BlockWriter,
    ) -> Result<(), Error> {
        bw.write_u32(ROOT_BLOCK_EXTENSION_OFFSET, self.extension)?;
        Ok(())
    }

    pub fn try_write_to_disk(
        &self,
        disk: &mut Disk,
    ) -> Result<(), Error> {
        let addr = disk.block_count()/2;
        let mut writer = BlockWriter::try_from_disk(disk, addr)?;

        self.try_write_bitmap(&mut writer)?;
        self.try_write_hash_table(&mut writer)?;
        self.try_write_volume_name(&mut writer)?;
        self.try_write_disk_alteration_date(&mut writer)?;
        self.try_write_root_alteration_date(&mut writer)?;
        self.try_write_root_creation_date(&mut writer)?;
        self.try_write_extension(&mut writer)?;

        writer.write_block_primary_type(BlockPrimaryType::Header)?;
        writer.write_block_secondary_type(BlockSecondaryType::RootDir)?;
        writer.write_checksum()?;

        Ok(())
    }
}
