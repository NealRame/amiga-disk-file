use std::path::Path;
use std::path::PathBuf;

use crate::disk::Disk;
use crate::errors::Error;

use super::block::*;
use super::boot_block::*;
use super::root_block::*;

use super::options::*;
use super::read_dir::*;


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

    pub fn root_block(&self) -> Result<RootBlock, Error> {
        let mut root_block = RootBlock::default();

        root_block.read(&self.disk)?;
        Ok(root_block)
    }

    pub fn boot_block(&self) -> Result<BootBlock, Error> {
        let mut boot_block = BootBlock::default();

        boot_block.read(&self.disk)?;
        Ok(boot_block)
    }
}

/******************************************************************************
* AmigaDos ReadDir ************************************************************
******************************************************************************/
impl AmigaDos {
    pub fn read_dir<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<ReadDir, Error> {
        ReadDir::try_from_disk(
            self.disk(),
            880,
            PathBuf::from(path.as_ref())
        )
    }
}

/******************************************************************************
* AmigaDosFormater ************************************************************
******************************************************************************/

#[derive(Clone, Debug, Default)]
pub struct AmigaDosFormater {
    filesystem_type: FilesystemType,
    filesystem_cache_mode: CacheMode,
    filesystem_intl_mode: InternationalMode,
}

impl AmigaDosFormater {
    pub fn width_filesystem_type(
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
        self.filesystem_intl_mode = international_mode;
        self
    }

    pub fn with_cache_mode(
        &mut self,
        cache_mode: CacheMode,
    ) -> &mut Self {
        self.filesystem_cache_mode = cache_mode;
        self
    }

    pub fn format(
        &self,
        mut disk: Disk,
        volume_name: &str,
    ) -> Result<AmigaDos, Error> {
        BootBlockBuilder::new(disk.disk_type())
            .width_filesystem_type(self.filesystem_type)
            .with_cache_mode(self.filesystem_cache_mode)
            .with_international_mode(self.filesystem_intl_mode)
            .build()
            .write(&mut disk)?;

        RootBlock::with_volume_name(volume_name).write(&mut disk)?;

        Ok(AmigaDos {
            disk
        })
    }
}
