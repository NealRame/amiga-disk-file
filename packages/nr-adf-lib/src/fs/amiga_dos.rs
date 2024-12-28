use std::time::SystemTime;

use crate::disk::Disk;
use crate::errors::Error;

use super::block::*;
use super::boot_block::*;
use super::root_block::*;
use super::options::*;


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
}

/******************************************************************************
* AmigaDosInfo ****************************************************************
******************************************************************************/

#[derive(Clone, Debug)]
pub struct AmigaDosInfo {
    pub volume_name: String,
    pub filesystem_type: FilesystemType,
    pub cache_mode: CacheMode,
    pub international_mode: InternationalMode,
    pub root_alteration_date: SystemTime,
    pub root_creation_date: SystemTime,
    pub volume_alteration_date: SystemTime,
}

impl AmigaDos {
    pub fn info(&self) -> Result<AmigaDosInfo, Error> {
        let boot_block = self.get_boot_block()?;
        let root_block = self.get_root_block()?;

        Ok(AmigaDosInfo {
            filesystem_type: boot_block.get_filesystem_type(),
            cache_mode: boot_block.get_cache_mode(),
            international_mode: boot_block.get_international_mode(),
            root_alteration_date: root_block.root_alteration_date,
            root_creation_date: root_block.root_creation_date,
            volume_alteration_date: root_block.volume_alteration_date,
            volume_name: root_block.volume_name,
        })
    }
}
