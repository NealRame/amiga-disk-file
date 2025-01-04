use std::time::SystemTime;

use crate::errors::*;

use super::amiga_dos::*;
use super::block::*;
use super::options::*;


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
        let root_block = BlockReader::try_from_disk(
            self.disk(),
            boot_block.get_root_block_address(),
        )?;

        let root_alteration_date = root_block.read_alteration_date()?;
        let root_creation_date = root_block.read_root_creation_date()?;
        let volume_alteration_date = root_block.read_disk_alteration_date()?;
        let volume_name = root_block.read_name()?;

        Ok(AmigaDosInfo {
            filesystem_type: boot_block.get_filesystem_type(),
            cache_mode: boot_block.get_cache_mode(),
            international_mode: boot_block.get_international_mode(),
            root_alteration_date,
            root_creation_date,
            volume_alteration_date,
            volume_name,
        })
    }
}
