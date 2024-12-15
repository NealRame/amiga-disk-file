use std::fmt;
use std::str::FromStr;

use crate::disk::{
    BLOCK_SIZE,
    Disk,
    DiskType,
};
use crate::errors::Error;

use super::constants::*;


#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum FilesystemType {
    OFS = 0,
    FFS = 0x01,
}

impl Default for FilesystemType {
    fn default() -> Self {
        FilesystemType::OFS
    }
}

impl FromStr for FilesystemType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ofs" => Ok(FilesystemType::OFS),
            "ffs" => Ok(FilesystemType::FFS),
            _ => Err(Error::InvalidFilesystemTypeError),
        }
    }
}

impl fmt::Display for FilesystemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilesystemType::OFS => write!(f, "OFS"),
            FilesystemType::FFS => write!(f, "FFS"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum InternationalMode {
    Off = 0,
    On  = 0x02,
}

impl Default for InternationalMode {
    fn default() -> Self {
        InternationalMode::Off
    }
}

impl FromStr for InternationalMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "on"|"yes" => Ok(InternationalMode::On),
            "off"|"no" => Ok(InternationalMode::Off),
            _ => Err(Error::InvalidInternationalModeError)
        }
    }
}

impl fmt::Display for InternationalMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternationalMode::On => write!(f, "INTL-ON"),
            InternationalMode::Off =>  write!(f, "INTL-OFF"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum CacheMode {
    Off  = 0,
    On = 0x04,
}

impl Default for CacheMode {
    fn default() -> Self {
        CacheMode::Off
    }
}

impl FromStr for CacheMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "on"|"yes" => Ok(CacheMode::On),
            "off"|"no" => Ok(CacheMode::Off),
            _ => Err(Error::InvalidCacheModeError)
        }
    }
}

impl fmt::Display for CacheMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheMode::On => write!(f, "CACHE-ON"),
            CacheMode::Off =>  write!(f, "CACHE-OFF"),
        }
    }
}

fn compute_checksum(data: &[u8]) -> u32 {
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

    !checksum
}

fn verify_checksum(data: &[u8], expected: u32) -> Result<(), Error> {
    if compute_checksum(data) != expected {
        Err(Error::CorruptedImageFile)
    } else {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct BootBlock {
    boot_code: [u8; BOOT_BLOCK_BOOT_CODE_SIZE],
    flags: u8,
    root_block_address: u32,
}

impl Default for BootBlock {
    fn default() -> Self {
        Self {
            boot_code: [0; BOOT_BLOCK_BOOT_CODE_SIZE],
            flags: 0,
            root_block_address: 0,
        }
    }
}

impl BootBlock {
    pub fn filesystem_type(&self) -> FilesystemType {
        if self.flags & 0x01 == 0 {
            FilesystemType::OFS
        } else {
            FilesystemType::FFS
        }
    }

    pub fn international_mode(&self) -> InternationalMode {
        if self.flags & 0x02 == 0 {
            InternationalMode::Off
        } else {
            InternationalMode::On
        }
    }

    pub fn cache_mode(&self) -> CacheMode {
        if self.flags & 0x04 == 0 {
            CacheMode::Off
        } else {
            CacheMode::On
        }
    }

    pub fn boot_code(&self) -> &[u8] {
        self.boot_code.as_slice()
    }

    pub fn root_block_address(&self) -> usize {
        self.root_block_address as usize
    }
}

impl BootBlock {
    pub fn try_read_from_disk(&mut self, disk: &Disk) -> Result<(), Error> {
        let mut data = disk.read_blocks(0, 2)?;

        if &data[0..3] != &[0x44, 0x4f, 0x53] { // DOS
            return Err(Error::CorruptedImageFile);
        }

        let checksum = u32::from_be_bytes(data[4..8].try_into().unwrap());

        data[4..8].fill(0);
        verify_checksum(&data, checksum)?;

        self.flags = data[3];
        self.boot_code.copy_from_slice(&data[12..]);
        self.root_block_address = u32::from_be_bytes(data[8..12].try_into().unwrap());

        Ok(())
    }
}

impl BootBlock {
    pub fn try_write_to_disk(&self, disk: &mut Disk) -> Result<(), Error> {
        let mut data = [0u8; 2*BLOCK_SIZE];

        data[BOOT_BLOCK_DISK_TYPE_SLICE].copy_from_slice(
            &[0x44, 0x4f, 0x53, self.flags],
        );
        data[BOOT_BLOCK_ROOT_BLOCK_SLICE].copy_from_slice(
            &self.root_block_address.to_be_bytes(),
        );
        data[BOOT_BLOCK_BOOT_CODE_SLICE].copy_from_slice(
            &self.boot_code,
        );

        let checksum = compute_checksum(&data);

        data[BOOT_BLOCK_CHECKSUM_SLICE].copy_from_slice(
            &checksum.to_be_bytes(),
        );

        disk.block_mut(0)?.copy_from_slice(&data[..BLOCK_SIZE]);
        disk.block_mut(1)?.copy_from_slice(&data[BLOCK_SIZE..]);

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BootBlockBuilder {
    boot_code: [u8; BOOT_BLOCK_BOOT_CODE_SIZE],
    flags: u8,
    root_block_address: u32,
}

impl BootBlockBuilder {
    pub fn new(disk_type: DiskType) -> Self {
        let boot_code = [0u8; BOOT_BLOCK_BOOT_CODE_SIZE];
        let root_block_address = ((disk_type as usize)/2) as u32;
        let flags = 0;

        Self {
            boot_code,
            flags,
            root_block_address,
        }
    }

    pub fn width_filesystem_type(
        &mut self,
        filesystem_type: FilesystemType,
    ) -> &mut Self {
        self.flags |= filesystem_type as u8;
        self
    }

    pub fn with_international_mode(
        &mut self,
        international_mode: InternationalMode,
    )-> &mut Self {
        self.flags |= international_mode as u8;
        self
    }

    pub fn with_cache_mode(
        &mut self,
        cache_mode: CacheMode,
    ) -> &mut Self {
        self.flags |= cache_mode as u8;
        self
    }

    pub fn build(self) -> BootBlock {
        BootBlock {
            boot_code: self.boot_code,
            flags: self.flags,
            root_block_address: self.root_block_address,
        }
    }
}
