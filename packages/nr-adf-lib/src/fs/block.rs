use std::time::SystemTime;

use paste::paste;

use crate::disk::{
    Disk,
    LBAAddress, BLOCK_SIZE,
};

use crate::errors::Error;

use super::block_type::*;
use super::checksum::*;
use super::constants::*;
use super::datetime::*;
use super::options::*;
use super::name::*;


/******************************************************************************
* BlockReader *****************************************************************
******************************************************************************/

pub struct BlockReader<'disk> {
    disk: &'disk Disk,
    data: &'disk [u8]
}

impl<'disk> BlockReader<'disk> {
    pub fn try_from_disk(
        disk: &'disk Disk,
        addr: LBAAddress,
    ) -> Result<Self, Error> {
        let data = disk.block(addr)?;

        Ok(Self {
            disk,
            data,
        })
    }
}

macro_rules! generate_read_fns {
    ($($t:ty),*) => {

        paste! {$(impl BlockReader<'_> {
            pub fn [<read_ $t>](
                &self,
                offset: usize,
            ) -> Result<$t, Error> {
                let size = std::mem::size_of::<$t>();

                if let Ok(buf) = self.data[offset..offset + size].try_into() {
                    Ok($t::from_be_bytes(buf))
                } else {
                    Err(Error::DiskInvalidBlockOffsetError(offset))
                }
            }

            pub fn [<read_ $t _array>](
                &self,
                offset: usize,
                values: &mut [$t],
            ) -> Result<(), Error> {
                let size = std::mem::size_of::<$t>();

                for i in 0..values.len() {
                    values[i] = self.[<read_ $t>](offset + i*size)?
                }
                Ok(())
            }

            pub fn [<read_ $t _vector>](
                &self,
                offset: usize,
                len: usize,
            ) -> Result<Vec<$t>, Error> {
                let mut v = Vec::new();

                v.resize(len, 0);
                self.[<read_ $t _array>](offset, &mut v)?;
                Ok(v)
            }
        })*}
    };
}

generate_read_fns!(u32);

impl BlockReader<'_> {
    pub fn read_u8(
        &self,
        offset: usize,
    ) -> Result<u8, Error> {
        if offset < self.data.len() {
            Ok(self.data[offset])
        } else {
            Err(Error::DiskInvalidBlockOffsetError(offset))
        }
    }

    pub fn read_u8_array(
        &self,
        offset: usize,
        data: &mut [u8],
    ) -> Result<(), Error> {
        if offset + data.len() <= self.data.len() {
            data.copy_from_slice(&self.data[offset..offset + data.len()]);
            Ok(())
        } else {
            Err(Error::DiskInvalidBlockOffsetError(offset))
        }
    }

    pub fn read_u8_vector(
        &self,
        offset: usize,
        len: usize,
    ) -> Result<Vec<u8>, Error> {
        let mut v = Vec::new();

        v.resize(len, 0);
        self.read_u8_array(offset, &mut v)?;
        Ok(v)
    }

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

    pub fn read_data_block_addr(
        &self,
        index: usize,
    ) -> Result<LBAAddress, Error> {
        self.check_block_primary_type(&[
            BlockPrimaryType::Header,
            BlockPrimaryType::List,
        ])?;
        self.check_block_secondary_type(&[
            BlockSecondaryType::File
        ])?;

        if index < BLOCK_BLOCK_DATA_LIST_SIZE {
            let addr = self.read_u32(BLOCK_BLOCK_DATA_LIST_OFFSET + 4*index)?;
            Ok(addr as LBAAddress)
        } else {
            Err(Error::InvalidDataBlockIndexError(index))
        }
    }

    pub fn read_data_extension_block_addr(
        &self,
    ) -> Result<LBAAddress, Error> {
        self.check_block_primary_type(&[
            BlockPrimaryType::Header,
            BlockPrimaryType::List,
        ])?;
        self.check_block_secondary_type(&[
            BlockSecondaryType::File
        ])?;

        let addr = self.read_u32(BLOCK_BLOCK_DATA_EXTENSION_OFFSET)? as usize;

        Ok(addr)
    }

    pub fn read_string(
        &self,
        offset: usize,
        len: usize,
    ) -> Result<String, Error> {
        let bytes = self.read_u8_vector(offset, len)?;

        if let Ok(s) = String::from_utf8(bytes) {
            Ok(s)
        } else {
            Err(Error::InvalidStringError)
        }
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

    // pub fn verify_checksum(&self, offset: usize) -> Result<(), Error> {
    //     let expected_checksum = self.read_u32(offset)?;

    //     if compute_checksum(self.data, offset) != expected_checksum {
    //         Err(Error::CorruptedImageFile)
    //     } else {
    //         Ok(())
    //     }
    // }
}

impl BlockReader<'_> {
    pub fn lookup(
        &self,
        name: &str,
        international_mode: InternationalMode,
    ) -> Result<Option<LBAAddress>, Error> {
        let hash_table = self.read_hash_table()?;
        let hash_index = hash_name(&name, international_mode);
        let mut addr = hash_table[hash_index] as LBAAddress;

        while addr != 0 {
            let br = BlockReader::try_from_disk(self.disk, addr)?;
            let entry_name = br.read_name()?;

            if entry_name == name {
                return Ok(Some(addr));
            }

            addr = br.read_u32(BLOCK_HASH_CHAIN_NEXT_OFFSET)? as LBAAddress;
        }

        Ok(None)
    }
}

fn non_zero(addr: &u32) -> bool {
    *addr != 0
}

impl BlockReader<'_> {
    pub fn read_dir(
        &self,
    ) -> Result<Vec<LBAAddress>, Error> {
        let mut entries = vec![];
        let hash_table = self.read_hash_table()?;

        for mut block_addr in hash_table.iter().copied().filter(non_zero) {
            while block_addr != 0 {
                entries.push(block_addr as LBAAddress);
                let br = BlockReader::try_from_disk(
                    self.disk,
                    block_addr as LBAAddress
                )?;

                block_addr = br.read_u32(BLOCK_HASH_CHAIN_NEXT_OFFSET)?;
            }
        }
        Ok(entries)
    }
}

/******************************************************************************
* BlockWriter *****************************************************************
******************************************************************************/

pub struct BlockWriter<'disk> {
    data: &'disk mut [u8]
}

impl<'disk> BlockWriter<'disk> {
    pub fn try_from_disk(
        disk: &'disk mut Disk,
        addr: LBAAddress,
    ) -> Result<Self, Error> {
        let data = disk.block_mut(addr)?;
        Ok(Self { data })
    }
}

macro_rules! generate_write_fns {
    ($($t:ty),*) => {

        paste! {$(impl BlockWriter<'_> {
            pub fn [<write_ $t>](
                &mut self,
                offset: usize,
                value: $t,
            ) -> Result<(), Error> {
                let size = std::mem::size_of::<$t>();
                let end = offset + size;

                if end <= self.data.len() {
                    let slice = &mut self.data[offset..end];
                    slice.copy_from_slice(&value.to_be_bytes());
                    Ok(())
                } else {
                    Err(Error::DiskInvalidBlockOffsetError(offset))
                }
            }

            // pub fn [<write_ $t _array>](
            //     &mut self,
            //     offset: usize,
            //     values: &[$t],
            // ) -> Result<(), Error> {
            //     for i in 0..values.len() {
            //         self.[<write_ $t>](offset + i, values[i])?
            //     }
            //     Ok(())
            // }
        })*}
    };
}

generate_write_fns!(u32);

impl BlockWriter<'_> {
    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    pub fn write_u8(
        &mut self,
        offset: usize,
        value: u8,
    ) -> Result<(), Error> {
        if offset < self.data.len() {
            self.data[offset] = value;
            Ok(())
        } else {
            Err(Error::DiskInvalidBlockOffsetError(offset))
        }
    }

    pub fn write_u8_array(
        &mut self,
        offset: usize,
        values: &[u8],
    ) -> Result<(), Error> {
        let size = values.len();

        if offset + size <= self.data.len() {
            self.data[offset..offset + size].copy_from_slice(values);
            Ok(())
        } else {
            Err(Error::DiskInvalidBlockOffsetError(offset))
        }
    }

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

    pub fn write_checksum(&mut self, offset: usize) -> Result<(), Error> {
        self.write_u32(offset, compute_checksum(self.data, offset))?;
        Ok(())
    }
}
