use crate::disk::*;
use crate::errors::*;

use super::block::BlockReader;
use super::boot_block::*;
use super::options::*;


pub struct AmigaDos {
    disk: Disk,
    bitmap_block_addresses: Box<[LBAAddress]>,
    root_block_address: LBAAddress,
}

impl TryFrom<Disk> for AmigaDos {
    type Error = Error;

    fn try_from(disk: Disk) -> Result<Self, Self::Error> {
        let boot_block = BootBlockReader::try_from_disk(&disk)?;

        let root_block_address = boot_block.get_root_block_address();
        let bitmap_block_addresses = BlockReader::try_from_disk(
            &disk,
            root_block_address,
        )?.read_bitmap()?.into_boxed_slice();

        Ok(Self {
            disk,
            bitmap_block_addresses,
            root_block_address,
        })
    }
}

impl AmigaDos {
    pub fn disk(&self) -> &Disk {
        &self.disk
    }

    pub fn disk_mut(&mut self) -> &mut Disk {
        &mut self.disk
    }

    pub fn get_boot_block(&self) -> Result<BootBlockReader, Error> {
        BootBlockReader::try_from_disk(self.disk())
    }

    pub fn get_filesystem_type(&self) -> Result<FilesystemType, Error> {
        Ok(self.get_boot_block()?.get_filesystem_type())
    }

    pub fn get_root_block_address(&self) -> LBAAddress {
        self.root_block_address
    }

    pub fn get_bitmap_block_addresses(&self) -> Vec<LBAAddress> {
        let mut addresses = Vec::new();
        addresses.extend_from_slice(&self.bitmap_block_addresses);
        addresses
    }
}
