use std::path::Path;
use std::cell::RefCell;
use std::rc::Rc;

use crate::block::*;
use crate::disk::*;
use crate::errors::*;

use super::amiga_dos_options::*;
use super::boot_block::*;


#[derive(Debug)]
pub(super) struct AmigaDosInner {
    disk: Rc<RefCell<Disk>>,
    bitmap_block_addresses: Box<[LBAAddress]>,
    // root_block_address: LBAAddress,
}

impl AmigaDosInner {
    pub(super) fn disk(&self) -> Rc<RefCell<Disk>> {
        self.disk.clone()
    }

    pub(super) fn get_boot_block(&self) -> Result<BootBlockReader, Error> {
        BootBlockReader::try_from_disk(self.disk())
    }

    pub(super) fn get_filesystem_type(&self) -> Result<FilesystemType, Error> {
        Ok(self.get_boot_block()?.get_filesystem_type())
    }

    // pub(super) fn get_root_block_address(&self) -> LBAAddress {
    //     self.root_block_address
    // }

    pub(super) fn get_bitmap_block_addresses(&self) -> Vec<LBAAddress> {
        let mut addresses = Vec::new();
        addresses.extend_from_slice(&self.bitmap_block_addresses);
        addresses
    }

    fn dump<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), std::io::Error> {
        std::fs::write(path, self.disk.borrow().data())?;
        Ok(())
    }
}

pub struct AmigaDos {
    pub(super) inner: Rc<RefCell<AmigaDosInner>>
}

impl TryFrom<Rc<RefCell<Disk>>> for AmigaDos {
    type Error = Error;

    fn try_from(disk: Rc<RefCell<Disk>>) -> Result<Self, Self::Error> {
        let boot_block = BootBlockReader::try_from_disk(disk.clone())?;

        let root_block_address = boot_block.get_root_block_address();
        let root_block = Block::new(
            disk.clone(),
            root_block_address,
        );

        let bitmap_block_addresses = root_block.read_bitmap()?.into_boxed_slice();

        Ok(Self {
            inner: Rc::new(RefCell::new(AmigaDosInner {
                disk,
                bitmap_block_addresses,
                // root_block_address,
            }))
        })
    }
}

impl AmigaDos {
    pub(super) fn disk(&self) -> Rc<RefCell<Disk>> {
        self.inner.borrow().disk()
    }

    pub(super) fn get_filesystem_type(&self) -> Result<FilesystemType, Error> {
        self.inner.borrow().get_filesystem_type()
    }

    // pub(super) fn get_root_block_address(&self) -> LBAAddress {
    //     self.inner.borrow().get_root_block_address()
    // }
}

impl AmigaDos {
    pub fn dump<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), std::io::Error> {
        self.inner.borrow().dump(path)
    }
}

impl AmigaDos {
    pub fn to_address(addr: u32) -> Option<LBAAddress> {
        if addr != 0 {
            Some(addr as LBAAddress)
        } else {
            None
        }
    }
}
