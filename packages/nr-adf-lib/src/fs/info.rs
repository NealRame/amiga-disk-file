use std::time::SystemTime;

use crate::block::*;
use crate::disk::BLOCK_SIZE;
use crate::errors::*;

use super::amiga_dos::*;
use super::amiga_dos_options::*;
use super::boot_block::*;


#[derive(Clone, Debug)]
pub struct AmigaDosInfo {
    pub volume_name: String,

    pub filesystem_type: FilesystemType,
    pub cache_mode: CacheMode,
    pub international_mode: InternationalMode,

    pub root_alteration_date: SystemTime,
    pub root_creation_date: SystemTime,
    pub volume_alteration_date: SystemTime,

    pub total_block_count: usize,
    pub total_size: usize,

    pub free_block_count: usize,
    pub free_size: usize,
}

impl AmigaDos {
    pub fn info(&self) -> Result<AmigaDosInfo, Error> {
        let fs = self.inner.borrow();
        let disk = fs.disk();

        let boot_block = BootBlockReader::try_from_disk(disk.clone())?;
        let root_block_address = boot_block.get_root_block_address();

        let root_block = Block::new(
            disk.clone(),
            root_block_address,
        );

        let volume_name = root_block.read_name()?;

        let root_alteration_date = root_block.read_alteration_date()?;
        let root_creation_date = root_block.read_root_creation_date()?;
        let volume_alteration_date = root_block.read_disk_alteration_date()?;

        let total_block_count = fs.total_block_count();
        let free_block_count = fs.free_block_count();

        Ok(AmigaDosInfo {
            filesystem_type: boot_block.get_filesystem_type(),
            cache_mode: boot_block.get_cache_mode(),
            international_mode: boot_block.get_international_mode(),

            root_alteration_date,
            root_creation_date,

            volume_alteration_date,
            volume_name,

            total_block_count,
            total_size: total_block_count*BLOCK_SIZE,

            free_block_count,
            free_size: free_block_count*BLOCK_SIZE,
        })
    }
}
