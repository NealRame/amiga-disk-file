use std::path::Path;
use std::cell::RefCell;
use std::rc::Rc;

use crate::disk::*;
use crate::errors::*;

use super::block::BlockReader;
use super::boot_block::*;
use super::options::*;


pub(super) struct AmigaDosInner {
    disk: Disk,
    bitmap_block_addresses: Box<[LBAAddress]>,
    root_block_address: LBAAddress,
}

impl AmigaDosInner {
    pub(super) fn disk(&self) -> &Disk {
        &self.disk
    }

    pub(super) fn disk_mut(&mut self) -> &mut Disk {
        &mut self.disk
    }

    pub(super) fn get_boot_block(&self) -> Result<BootBlockReader, Error> {
        BootBlockReader::try_from_disk(self.disk())
    }

    pub(super) fn get_filesystem_type(&self) -> Result<FilesystemType, Error> {
        Ok(self.get_boot_block()?.get_filesystem_type())
    }

    pub(super) fn get_root_block_address(&self) -> LBAAddress {
        self.root_block_address
    }

    pub(super) fn get_bitmap_block_addresses(&self) -> Vec<LBAAddress> {
        let mut addresses = Vec::new();
        addresses.extend_from_slice(&self.bitmap_block_addresses);
        addresses
    }

    fn dump<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), std::io::Error> {
        std::fs::write(path, self.disk.data())?;
        Ok(())
    }
}

pub struct AmigaDos {
    pub(super) inner: Rc<RefCell<AmigaDosInner>>
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
            inner: Rc::new(RefCell::new(AmigaDosInner {
                disk,
                bitmap_block_addresses,
                root_block_address,
            }))
        })
    }
}

impl AmigaDos {
    pub fn dump<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), std::io::Error> {
        self.inner.borrow().dump(path)
    }
}
