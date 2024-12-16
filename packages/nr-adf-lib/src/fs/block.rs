use paste::paste;

use crate::disk::{
    Disk,
    LBAAddress, BLOCK_SIZE,
};

use crate::errors::Error;

use super::block_type::*;
use super::constants::*;

fn compute_checksum(data: &[u8]) -> u32 {
    let mut checksum = 0u32;

    for (i, chunk) in data.chunks(4).enumerate() {
        if chunk.len() == 4 {
            let d = u32::from_be_bytes(chunk.try_into().unwrap());

            if i != 5 {
                checksum = checksum.overflowing_add(d).0;
            }
        }
    }

    !checksum
}

/******************************************************************************
* BlockReader *****************************************************************
******************************************************************************/

pub struct BlockReader<'disk> {
    data: &'disk [u8]
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
        })*}
    };
}

generate_read_fns!(u32);

impl<'disk> BlockReader<'disk> {
    pub fn try_from_disk(
        disk: &'disk Disk,
        addr: LBAAddress,
    ) -> Result<Self, Error> {
        let data = disk.block(addr)?;
        Ok(Self { data })
    }
}

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
        v: &mut [u8],
    ) -> Result<(), Error> {
        if offset + v.len() <= self.data.len() {
            v.copy_from_slice(&self.data[offset..offset + v.len()]);
            Ok(())
        } else {
            Err(Error::DiskInvalidBlockOffsetError(offset))
        }
    }

    pub fn verify_block_primary_type(
        &self,
        expected_block_primary_type: BlockPrimaryType,
    ) -> Result<(), Error> {
        let v = self.read_u32(0)?;
        let block_primary_type = BlockPrimaryType::try_from(v)?;

        if block_primary_type != expected_block_primary_type {
            Err(Error::UnexpectedFilesystemBlockPrimaryTypeError(v))
        } else {
            Ok(())
        }
    }

    pub fn verify_block_secondary_type(
        &self,
        expected_block_secondary_type: BlockSecondaryType,
    ) -> Result<(), Error> {
        let v: u32 = self.read_u32(BLOCK_SIZE - 4)?;
        let block_secondary_type = BlockSecondaryType::try_from(v)?;

        if block_secondary_type != expected_block_secondary_type {
            Err(Error::UnexpectedFilesystemBlockSecondaryTypeError(v))
        } else {
            Ok(())
        }
    }

    pub fn verify_checksum(&self) -> Result<(), Error> {
        let expected_checksum = self.read_u32(BLOCK_CHECKSUM_OFFSET)?;

        if compute_checksum(self.data) != expected_checksum {
            Err(Error::CorruptedImageFile)
        } else {
            Ok(())
        }
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

            pub fn [<write_ $t _array>](
                &mut self,
                offset: usize,
                values: &[$t],
            ) -> Result<(), Error> {
                for i in 0..values.len() {
                    self.[<write_ $t>](offset + i, values[i])?
                }
                Ok(())
            }
        })*}
    };
}

generate_write_fns!(u32);

impl BlockWriter<'_> {
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

    pub fn write_checksum(&mut self) -> Result<(), Error> {
        self.write_u32(BLOCK_CHECKSUM_OFFSET, compute_checksum(self.data))?;
        Ok(())
    }
}
