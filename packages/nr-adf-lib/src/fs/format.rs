use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::block::*;
use super::boot_block::*;
use super::root_block::*;
use super::options::*;


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
            .with_filesystem_type(self.filesystem_type)
            .with_cache_mode(self.cache_mode)
            .with_international_mode(self.international_mode)
            .write(&mut disk)?;

        RootBlock::with_volume_name(volume_name).write(&mut disk)?;

        Ok(AmigaDos::from(disk))
    }
}
