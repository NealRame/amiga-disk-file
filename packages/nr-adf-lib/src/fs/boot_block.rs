use std::fmt;

use crate::disk::{
    BLOCK_SIZE,
    Disk,
};

use crate::errors::Error;


#[repr(u8)]
pub enum FileSystemType {
    OFS = 0,
    FFS = 1,
}

impl fmt::Display for FileSystemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileSystemType::OFS => write!(f, "OFS"),
            FileSystemType::FFS => write!(f, "FFS"),
        }
    }
}


#[repr(u8)]
pub enum InternationalMode {
    No  = 0,
    Yes = 1,
}

impl fmt::Display for InternationalMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternationalMode::Yes => write!(f, "INTL"),
            InternationalMode::No =>  write!(f, "NO_INTL"),
        }
    }
}

#[repr(u8)]
pub enum CacheMode {
    No  = 0,
    Yes = 1,
}

impl fmt::Display for CacheMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheMode::Yes => write!(f, "CACHE"),
            CacheMode::No =>  write!(f, "NO_CACHE"),
        }
    }
}

pub struct BootBlock {
    boot_code: [u8; 2*BLOCK_SIZE - 12],
    flags: u8,
    root_block_address: u32,
}

fn verify_checksum(data: &[u8], expected: u32) -> Result<(), Error> {
    let mut checksum = 0u32;

    for chunk in data.chunks(4) {
        if chunk.len() == 4 {
            let d = u32::from_be_bytes(chunk.try_into().unwrap());

            if u32::MAX < d {
                checksum = checksum.overflowing_add(1).0;
            }
            checksum = checksum.overflowing_add(d).0;
        }
    }

    if expected == !checksum {
        Err(Error::CorruptedImageFile)
    } else {
        Ok(())
    }
}

impl BootBlock {
    pub fn try_read_from_disk(disk: &Disk) -> Result<Self, Error> {
        let mut data = disk.read_blocks(0, 2)?;

        if &data[0..3] != &[0x44, 0x4f, 0x53] { // DOS
            return Err(Error::CorruptedImageFile);
        }

        let checksum = u32::from_be_bytes(data[4..8].try_into().unwrap());

        data[4..8].fill(0);
        verify_checksum(&data, checksum)?;

        let flags = data[3];
        let root_block_address = u32::from_be_bytes(data[8..12].try_into().unwrap());

        let mut boot_code = [0u8; 2*BLOCK_SIZE - 12];
        boot_code.copy_from_slice(&data[12..]);

        Ok(BootBlock {
            boot_code,
            flags,
            root_block_address,
        })
    }
}

impl BootBlock {
    pub fn filesystem_type(&self) -> FileSystemType {
        if self.flags & 0x01 == 0 {
            FileSystemType::OFS
        } else {
            FileSystemType::FFS
        }
    }


    pub fn international_mode(&self) -> InternationalMode {
        if self.flags & 0x02 == 0 {
            InternationalMode::No
        } else {
            InternationalMode::Yes

        }
    }

    pub fn cache_mode(&self) -> CacheMode {
        if self.flags & 0x04 == 0 {
            CacheMode::No
        } else {
            CacheMode::Yes
        }
    }

    pub fn boot_code(&self) -> &[u8] {
        self.boot_code.as_slice()
    }

    pub fn root_block_address(&self) -> usize {
        self.root_block_address as usize
    }
}
