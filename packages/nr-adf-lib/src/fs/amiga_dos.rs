use std::time::SystemTime;

use crate::disk::Disk;
use crate::errors::Error;

use super::block::*;
use super::boot_block::*;
use super::root_block::*;
use super::options::*;


pub struct AmigaDos {
    disk: Disk,
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

    fn root_block(&self) -> Result<RootBlock, Error> {
        let mut root_block = RootBlock::default();

        root_block.read(self.disk())?;
        Ok(root_block)
    }

    fn boot_block(&self) -> Result<BootBlock, Error> {
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
        let boot_block = self.boot_block()?;
        let root_block = self.root_block()?;

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

/******************************************************************************
* AmigaDosFormater ************************************************************
******************************************************************************/

#[derive(Clone, Debug, Default)]
pub struct AmigaDosFormater {
    filesystem_type: FilesystemType,
    cache_mode: CacheMode,
    international_mode: InternationalMode,
}

impl AmigaDosFormater {
    pub fn with_filesystem_type(
        &mut self,
        filesystem_type: FilesystemType,
    ) -> &mut Self {
        self.filesystem_type = filesystem_type;
        self
    }

    pub fn with_international_mode(
        &mut self,
        international_mode: InternationalMode,
    )-> &mut Self {
        self.international_mode = international_mode;
        self
    }

    pub fn with_cache_mode(
        &mut self,
        cache_mode: CacheMode,
    ) -> &mut Self {
        self.cache_mode = cache_mode;
        self
    }

    pub fn format(
        &self,
        mut disk: Disk,
        volume_name: &str,
    ) -> Result<AmigaDos, Error> {
        BootBlockWriter::default()
            .width_filesystem_type(self.filesystem_type)
            .with_cache_mode(self.cache_mode)
            .with_international_mode(self.international_mode)
            .write(&mut disk)?;

        RootBlock::with_volume_name(volume_name).write(&mut disk)?;

        Ok(AmigaDos {
            disk
        })
    }
}
