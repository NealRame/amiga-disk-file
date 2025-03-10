use std::time::SystemTime;

use crate::disk::*;

use crate::block::Block;
use crate::errors::Error;

use super::block_type::*;
use super::checksum::*;
use super::constants::*;
use super::datetime::*;
use super::name::*;
use super::AmigaDos;


impl Block {
    pub fn compute_checksum(
        &self,
    ) -> Result<u32, Error> {
        let disk = self.disk.borrow();
        let disk_data = disk.blocks(self.address, 1)?;

        Ok(compute_checksum(disk_data, BLOCK_SIZE))
    }
}


impl Block {
    pub fn read_block_primary_type(
        &self,
    ) -> Result<BlockPrimaryType, Error> {
        let v = self.read_u32(0)?;
        BlockPrimaryType::try_from(v)
    }

    pub fn read_block_secondary_type(
        &self,
    ) -> Result<BlockSecondaryType, Error> {
        let v: u32 = self.read_u32(BLOCK_SIZE - 4)?;
        BlockSecondaryType::try_from(v)
    }

    pub fn check_block_primary_type(
        &self,
        expected_block_primary_types: &[BlockPrimaryType],
    ) -> Result<(), Error> {
        let block_type = self.read_block_primary_type()?;

        for expected in expected_block_primary_types {
            if block_type == *expected {
                return Ok(())
            }
        }

        Err(Error::UnexpectedFilesystemBlockPrimaryTypeError(block_type as u32))
    }

    pub fn check_block_secondary_type(
        &self,
        expected_block_secondary_types: &[BlockSecondaryType],
    ) -> Result<(), Error> {
        let block_type = self.read_block_secondary_type()?;

        for expected in expected_block_secondary_types {
            if block_type == *expected {
                return Ok(())
            }
        }

        Err(Error::UnexpectedFilesystemBlockSecondaryTypeError(block_type as u32))
    }
}


impl Block {
    // TODO delete this function (only used for read dir)
    pub fn read_hash_table(
        &self,
    ) -> Result<Vec<u32>, Error> {
        self.check_block_primary_type(&[BlockPrimaryType::Header])?;
        self.check_block_secondary_type(&[
            BlockSecondaryType::Root,
            BlockSecondaryType::Directory,
        ])?;
        self.read_u32_vector(
            BLOCK_HASH_TABLE_OFFSET,
            BLOCK_HASH_TABLE_SIZE,
        )
    }

    pub fn read_bitmap(
        &self,
    ) -> Result<Vec<LBAAddress>, Error> {
        self.check_block_primary_type(&[BlockPrimaryType::Header])?;
        self.check_block_secondary_type(&[BlockSecondaryType::Root])?;

        let v = self.read_u32_vector(
            ROOT_BLOCK_BITMAP_PAGES_OFFSET,
            ROOT_BLOCK_BITMAP_PAGES_MAX_COUNT,
        )?;

        Ok(v.iter().copied().filter_map(|addr| {
            if addr != 0 {
                Some(addr as LBAAddress)
            } else {
                None
            }
        }).collect())
    }

    pub fn read_block_table_address(
        &mut self,
        index: usize,
    ) -> Result<Option<LBAAddress>, Error> {
        self.check_block_primary_type(&[
            BlockPrimaryType::Header,
            BlockPrimaryType::List,
        ])?;

        if index < BLOCK_TABLE_SIZE {
            Ok(AmigaDos::to_address(self.read_u32(BLOCK_TABLE_OFFSET + 4*index)?))
        } else {
            Err(Error::InvalidDataBlockIndexError(index))
        }
    }

    pub fn read_block_chain_next_address(
        &mut self,
    ) -> Result<Option<LBAAddress>, Error> {
        self.check_block_primary_type(&[
            BlockPrimaryType::Header,
            BlockPrimaryType::List,
        ])?;
        Ok(AmigaDos::to_address(self.read_u32(BLOCK_CHAIN_NEXT_OFFSET)?))
    }

    pub fn read_name(
        &self,
    ) -> Result<String, Error> {
        self.check_block_primary_type(&[BlockPrimaryType::Header])?;
        self.check_block_secondary_type(&[
            BlockSecondaryType::Directory,
            BlockSecondaryType::File,
            BlockSecondaryType::HardLinkDirectory,
            BlockSecondaryType::HardLinkFile,
            BlockSecondaryType::Root,
            BlockSecondaryType::SoftLink,
        ])?;

        let len = self.read_u8(BLOCK_NAME_SIZE_OFFSET)? as usize;

        if len <= BLOCK_NAME_MAX_SIZE {
            let name = self.read_string(BLOCK_NAME_OFFSET, len)?;

            check_name(&name)?;
            Ok(name)
        } else {
            Err(Error::InvalidNameLengthError(len))
        }
    }

    pub fn read_file_size(
        &self,
    ) -> Result<usize, Error> {
        self.check_block_primary_type(&[BlockPrimaryType::Header])?;
        self.check_block_secondary_type(&[BlockSecondaryType::File])?;

        let file_size = self.read_u32(BLOCK_FILE_SIZE)? as usize;

        Ok(file_size)
    }

    pub fn read_alteration_date(
        &self,
    ) -> Result<SystemTime, Error> {
        self.check_block_primary_type(&[BlockPrimaryType::Header])?;
        self.check_block_secondary_type(&[
            BlockSecondaryType::Directory,
            BlockSecondaryType::File,
            BlockSecondaryType::HardLinkFile,
            BlockSecondaryType::HardLinkDirectory,
            BlockSecondaryType::Root,
        ])?;

        let days = self.read_u32(BLOCK_ALTERATION_DAYS_OFFSET)?;
        let mins = self.read_u32(BLOCK_ALTERATION_MINS_OFFSET)?;
        let ticks = self.read_u32(BLOCK_ALTERATION_TICKS_OFFSET)?;

        Ok(date_triplet_to_system_time(days, mins, ticks))
    }

    pub fn read_disk_alteration_date(
        &self,
    ) -> Result<SystemTime, Error> {
        self.check_block_primary_type(&[BlockPrimaryType::Header])?;
        self.check_block_secondary_type(&[BlockSecondaryType::Root])?;

        let days = self.read_u32(ROOT_BLOCK_V_DAYS_OFFSET)?;
        let mins = self.read_u32(ROOT_BLOCK_V_MINS_OFFSET)?;
        let ticks = self.read_u32(ROOT_BLOCK_V_TICKS_OFFSET)?;

        Ok(date_triplet_to_system_time(days, mins, ticks))
    }

    pub fn read_root_creation_date(
        &self,
    ) -> Result<SystemTime, Error> {
        self.check_block_primary_type(&[BlockPrimaryType::Header])?;
        self.check_block_secondary_type(&[BlockSecondaryType::Root])?;

        let days = self.read_u32(ROOT_BLOCK_C_DAYS_OFFSET)?;
        let mins = self.read_u32(ROOT_BLOCK_C_MINS_OFFSET)?;
        let ticks = self.read_u32(ROOT_BLOCK_C_TICKS_OFFSET)?;

        Ok(date_triplet_to_system_time(days, mins, ticks))
    }
}

impl Block {
    pub fn write_alteration_date(
        &mut self,
        datetime: &SystemTime,
    ) -> Result<(), Error> {
        let (
            days,
            mins,
            ticks,
        ) = date_triplet_from_system_time(datetime);

        self.write_u32(BLOCK_ALTERATION_DAYS_OFFSET, days)?;
        self.write_u32(BLOCK_ALTERATION_MINS_OFFSET, mins)?;
        self.write_u32(BLOCK_ALTERATION_TICKS_OFFSET, ticks)?;

        Ok(())
    }

    pub fn write_disk_alteration_date(
        &mut self,
        datetime: &SystemTime,
    ) -> Result<(), Error> {
        let (
            days,
            mins,
            ticks,
        ) = date_triplet_from_system_time(datetime);

        self.write_u32(ROOT_BLOCK_V_DAYS_OFFSET, days)?;
        self.write_u32(ROOT_BLOCK_V_MINS_OFFSET, mins)?;
        self.write_u32(ROOT_BLOCK_V_TICKS_OFFSET, ticks)?;

        Ok(())
    }

    pub fn write_root_creation_date(
        &mut self,
        datetime: &SystemTime,
    ) -> Result<(), Error> {
        let (
            days,
            mins,
            ticks,
        ) = date_triplet_from_system_time(datetime);

        self.write_u32(ROOT_BLOCK_C_DAYS_OFFSET, days)?;
        self.write_u32(ROOT_BLOCK_C_MINS_OFFSET, mins)?;
        self.write_u32(ROOT_BLOCK_C_TICKS_OFFSET, ticks)?;

        Ok(())
    }

    pub fn write_block_primary_type(
        &mut self,
        block_primary_type: BlockPrimaryType,
    ) -> Result<(), Error> {
        self.write_u32(BLOCK_PRIMARY_TYPE_OFFSET, block_primary_type as u32)?;
        Ok(())
    }

    pub fn write_block_secondary_type(
        &mut self,
        block_secondary_type: BlockSecondaryType,
    ) -> Result<(), Error> {
        self.write_u32(BLOCK_SECONDARY_TYPE_OFFSET, block_secondary_type as u32)?;
        Ok(())
    }

    pub fn write_name(
        &mut self,
        name: &str,
    ) -> Result<(), Error> {
        check_name(&name)?;

        let bytes = name.as_bytes();
        let len = bytes.len();

        if len <= BLOCK_NAME_MAX_SIZE {
            self.write_u8(BLOCK_NAME_SIZE_OFFSET, len as u8)?;
            self.write_u8_array(BLOCK_NAME_OFFSET, &bytes)?;
            Ok(())
        } else {
            Err(Error::InvalidNameLengthError(len))
        }
    }

    pub fn write_checksum(
        &mut self,
        offset: usize,
    ) -> Result<(), Error> {
        let chksum = {
            let disk = self.disk.borrow();
            let disk_data = disk.blocks(self.address, 1)?;

            compute_checksum(disk_data, offset)
        };
        self.write_u32(offset, chksum)
    }

    // pub fn write_data_block_addr(
    //     &mut self,
    //     index: usize,
    //     address: LBAAddress,
    // ) -> Result<(), Error> {
    //     if index < BLOCK_DATA_LIST_SIZE {
    //         self.write_u32(
    //             BLOCK_DATA_LIST_OFFSET + 4*index,
    //             address as u32,
    //         )?;
    //         Ok(())
    //     } else {
    //         Err(Error::InvalidDataBlockIndexError(index))
    //     }
    // }

    // pub fn write_data_extension_block_addr(
    //     &mut self,
    //     address: LBAAddress,
    // ) -> Result<(), Error> {
    //     self.write_u32(BLOCK_DATA_LIST_EXTENSION_OFFSET, address as u32)
    // }

    pub fn write_block_table_address(
        &mut self,
        index: usize,
        address: LBAAddress,
    ) -> Result<(), Error> {
        if index < BLOCK_TABLE_SIZE {
            self.write_u32(
                BLOCK_TABLE_OFFSET + 4*index,
                address as u32,
            )?;
            Ok(())
        } else {
            Err(Error::InvalidDataBlockIndexError(index))
        }
    }

    pub fn write_block_chain_next_address(
        &mut self,
        address: LBAAddress,
    ) -> Result<(), Error> {
        self.write_u32(BLOCK_CHAIN_NEXT_OFFSET, address as u32)
    }
}
