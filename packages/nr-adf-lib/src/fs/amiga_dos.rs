use crate::disk::Disk;
use crate::errors::Error;

use super::boot_block::*;
use super::root_block::*;


pub struct AmigaDos {
    disk: Disk,
}

impl TryFrom<Disk> for AmigaDos {
    type Error = Error;

    fn try_from(disk: Disk) -> Result<Self, Self::Error> {
        let mut boot_block = BootBlock::default();
        let mut root_block = RootBlock::default();

        boot_block.try_read_from_disk(&disk)?;
        root_block.try_read_from_disk(&disk)?;

        Ok(AmigaDos { disk })
    }
}

impl AmigaDos {
    pub fn disk(&self) -> &Disk {
        &self.disk
    }

    pub fn root_block(&self) -> Result<RootBlock, Error> {
        let mut root_block = RootBlock::default();

        root_block.try_read_from_disk(&self.disk)?;
        Ok(root_block)
    }

    pub fn boot_block(&self) -> Result<BootBlock, Error> {
        let mut boot_block = BootBlock::default();

        boot_block.try_read_from_disk(&self.disk)?;
        Ok(boot_block)
    }
}

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
        let boot_block =
            BootBlockBuilder::new(disk.disk_type())
                .width_filesystem_type(self.filesystem_type)
                .with_cache_mode(self.filesystem_cache_mode)
                .with_international_mode(self.filesystem_intl_mode)
                .build();
        boot_block.try_write_to_disk(&mut disk)?;

        let mut root_block = RootBlock::default();
        root_block.volume_name = volume_name.into();
        root_block.try_write_to_disk(&mut disk)?;

        Ok(AmigaDos {
            disk
        })
    }
}
