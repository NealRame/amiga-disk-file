use crate::disk::{
    Disk,
    LBAAddress, BLOCK_SIZE,
};

use crate::errors::Error;
use crate::read::*;

use super::block_type::*;

pub struct BlockReader<'disk> {
    data: &'disk [u8]
}

fn verify_checksum(data: &[u8]) -> Result<(), Error> {
    let expected = u32::from_be_bytes(data[20..24].try_into().unwrap());
    let mut checksum = 0u32;

    for (i, chunk) in data.chunks(4).enumerate() {
        if chunk.len() == 4 {
            let d = u32::from_be_bytes(chunk.try_into().unwrap());

            if i != 5 {
                checksum = checksum.overflowing_add(d).0;
            }
        }
    }

    if expected == !checksum {
        Err(Error::CorruptedImageFile)
    } else {
        Ok(())
    }
}

impl<'disk> BlockReader<'disk> {
    pub fn try_from_disk(
        disk: &'disk Disk,
        addr: LBAAddress,
    ) -> Result<Self, Error> {
        let data = disk.block(addr)?;
        verify_checksum(&data)?;
        Ok(Self { data })
    }
}

impl BlockReader<'_> {
    pub fn read_u8(
        &self,
        offset: usize,
        v: &mut u8,
    ) -> Result<(), Error> {
        if offset < self.data.len() {
            *v = self.data[offset];
            Ok(())
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

    pub fn read_u32(
        &self,
        offset: usize,
        v: &mut u32,
    ) -> Result<(), Error> {
        if offset + 4 <= self.data.len() {
            *v = u32::try_read_from::<MSB>(&mut &self.data[offset..]).unwrap();
            Ok(())
        } else {
            Err(Error::DiskInvalidBlockOffsetError(offset))
        }
    }

    pub fn read_u32_array(
        &self,
        offset: usize,
        v: &mut [u32],
    ) -> Result<(), Error> {
        for i in 0..v.len() {
            self.read_u32(offset + i, &mut v[i])?;
        }
        Ok(())
    }

    pub fn read_block_primary_type(
        &self,
        block_primary_type: &mut BlockPrimaryType,
    ) -> Result<(), Error> {
        let mut v: u32 = 0;

        self.read_u32(0, &mut v)?;
        *block_primary_type = BlockPrimaryType::try_from(v)?;
        Ok(())
    }

    pub fn read_block_secondary_type(
        &self,
        block_secondary_type: &mut BlockSecondaryType,
    ) -> Result<(), Error> {
        let mut v: u32 = 0;

        self.read_u32(BLOCK_SIZE - 4, &mut v)?;
        *block_secondary_type = BlockSecondaryType::try_from(v)?;
        Ok(())
    }
}
