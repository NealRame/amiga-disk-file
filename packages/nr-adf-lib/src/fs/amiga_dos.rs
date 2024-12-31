use crate::disk::Disk;
use crate::errors::Error;

use super::block::*;
use super::boot_block::*;
use super::root_block::*;
use super::FilesystemType;


pub struct AmigaDos {
    pub disk: Disk,
}

impl From<Disk> for AmigaDos {
    fn from(disk: Disk) -> Self {
        AmigaDos { disk }
    }
}

impl AmigaDos {
    pub fn disk(&self) -> &Disk {
        &self.disk
    }

    pub fn get_root_block(&self) -> Result<RootBlock, Error> {
        let mut root_block = RootBlock::default();

        root_block.read(self.disk())?;
        Ok(root_block)
    }

    pub fn get_boot_block(&self) -> Result<BootBlock, Error> {
        BootBlock::try_from_disk(self.disk())
    }

    pub fn get_filesystem_type(&self) -> Result<FilesystemType, Error> {
        Ok(self.get_boot_block()?.get_filesystem_type())
    }
}
