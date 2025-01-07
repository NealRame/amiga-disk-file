use crate::disk::Disk;
use crate::errors::Error;

use super::boot_block::*;
use super::options::*;


pub struct AmigaDos {
    pub disk: Disk,
}

impl TryFrom<Disk> for AmigaDos {
    type Error = Error;

    fn try_from(disk: Disk) -> Result<Self, Self::Error> {
        Ok(Self {
            disk
        })
    }
}


impl AmigaDos {
    pub fn disk(&self) -> &Disk {
        &self.disk
    }

    pub fn get_boot_block(&self) -> Result<BootBlock, Error> {
        BootBlock::try_from_disk(self.disk())
    }

    pub fn get_filesystem_type(&self) -> Result<FilesystemType, Error> {
        Ok(self.get_boot_block()?.get_filesystem_type())
    }
}
