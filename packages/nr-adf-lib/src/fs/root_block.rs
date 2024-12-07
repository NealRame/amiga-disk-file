use core::str;
use std::time::{
    UNIX_EPOCH,
    SystemTime,
};

use crate::disk::*;
use crate::errors::*;

use super::block::*;
use super::block_type::*;
use super::constants::*;


#[derive(Clone, Debug)]
pub struct RootBlock {
    pub block_primary_type: BlockPrimaryType,
    pub block_secondary_type: BlockSecondaryType,

    pub volume_name: String,

    pub hash_table_size: u32,
    pub hash_table: [u32; ROOT_BLOCK_HASH_TABLE_MAX_SIZE],

    pub bitmap_flag: u32,
    pub bitmap_pages: [u32; ROOT_BLOCK_BITMAP_MAX_PAGES],
    pub bitmap_ext: u32,

    // last root alteration date
    pub r_days: u32,
    pub r_mins: u32,
    pub r_ticks: u32,

    // last disk alteration date
    pub v_days: u32,
    pub v_mins: u32,
    pub v_ticks: u32,

    // filesystem creation date
    pub c_days: u32,
    pub c_mins: u32,
    pub c_ticks: u32,

    pub extension: u32,
}

impl Default for RootBlock {
    fn default() -> Self {
        // seconds since epoch
        let seconds = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("SystemTime before UNIX EPOCH!")
                .as_secs();

        // days since epoch
        let r_days = seconds/SECONDS_PER_DAY;

        // mins past midnight
        let r_mins = (seconds - r_days*SECONDS_PER_DAY)/SECONDS_PER_MINUTE;

        // ticks past last minute
        let r_ticks = (seconds - r_mins*SECONDS_PER_MINUTE)/50;

        let mut disk_name = [0u8; ROOT_BLOCK_DISK_NAME_MAX_SIZE];

        disk_name[0] = 0x44; // D
        disk_name[1] = 0x49; // I
        disk_name[2] = 0x53; // S
        disk_name[4] = 0x4b; // K

        return Self {
            block_primary_type: BlockPrimaryType::Header,
            block_secondary_type: BlockSecondaryType::RootDir,

            hash_table_size: 0,
            hash_table: [0u32; ROOT_BLOCK_HASH_TABLE_MAX_SIZE],

            bitmap_flag: 0,
            bitmap_pages: [0u32; ROOT_BLOCK_BITMAP_MAX_PAGES],
            bitmap_ext: 0,

            volume_name: String::from("VOLUME"),

            r_days: r_days as u32,
            r_mins: r_mins as u32,
            r_ticks: r_ticks as u32,

            v_days: r_days as u32,
            v_mins: r_mins as u32,
            v_ticks: r_ticks as u32,

            c_days: r_days as u32,
            c_mins: r_mins as u32,
            c_ticks: r_ticks as u32,

            extension: 0,
        }
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
        br.read_u32(ROOT_BLOCK_R_DAYS_OFFSET, &mut self.r_days)?;
        br.read_u32(ROOT_BLOCK_R_MINS_OFFSET, &mut self.r_mins)?;
        br.read_u32(ROOT_BLOCK_R_TICKS_OFFSET, &mut self.r_ticks)?;
        Ok(())
    }

    fn try_read_disk_alteration_date(
        &mut self,
        br: &BlockReader,
    ) -> Result<(), Error> {
        br.read_u32(ROOT_BLOCK_V_DAYS_OFFSET, &mut self.v_days)?;
        br.read_u32(ROOT_BLOCK_V_MINS_OFFSET, &mut self.v_mins)?;
        br.read_u32(ROOT_BLOCK_V_TICKS_OFFSET, &mut self.v_ticks)?;
        Ok(())
    }

    fn try_read_fs_creation_date(
        &mut self,
        br: &BlockReader,
    ) -> Result<(), Error> {
        br.read_u32(ROOT_BLOCK_C_DAYS_OFFSET, &mut self.c_days)?;
        br.read_u32(ROOT_BLOCK_C_MINS_OFFSET, &mut self.c_mins)?;
        br.read_u32(ROOT_BLOCK_C_TICKS_OFFSET, &mut self.c_ticks)?;
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
        self.try_read_fs_creation_date(&reader)?;

        self.try_read_extension(&reader)?;

        Ok(())
    }
}
