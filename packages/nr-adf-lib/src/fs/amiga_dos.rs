use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::disk::Disk;
use crate::disk::LBAAddress;
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

    fn root_block(&self) -> Result<RootBlock, Error> {
        let mut root_block = RootBlock::default();

        root_block.read(&self.disk)?;
        Ok(root_block)
    }
}

/******************************************************************************
* AmigaDos ReadDir ************************************************************
******************************************************************************/
fn split_path<P: AsRef<Path>>(
    path: P,
) -> Option<Vec<String>> {
    path.as_ref().to_str()
        .map(|path| path.split("/"))
        .map(|strs| strs.filter_map(|s| {
            if s.len() > 0 {
                Some(String::from(s))
            } else {
                None
            }
        }))
        .map(|res| res.collect::<Vec<String>>())
}


impl AmigaDos {
    fn lookup<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<LBAAddress, Error> {
        if let Some(path) = split_path(path) {
            let disk = self.disk();
            let boot_block = BootBlockReader::from_disk(disk)?;
            let international_mode = boot_block.international_mode;
            let mut block_addr = boot_block.root_block_address;

            for name in path {
                let br = BlockReader::try_from_disk(disk, block_addr)?;

                if let Some(addr) = br.lookup(&name, international_mode)? {
                    block_addr = addr;
                } else {
                    return Err(Error::NotFoundError);
                }
            }

            Ok(block_addr)
        } else {
            Err(Error::InvalidPathError)
        }
    }

    pub fn read_dir<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<ReadDir, Error> {
        let block_addr = self.lookup(&path)?;

        ReadDir::try_from_disk(
            self.disk(),
            block_addr,
            PathBuf::from(path.as_ref())
        )
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
        let boot_block = BootBlockReader::from_disk(self.disk())?;
        let root_block = self.root_block()?;

        Ok(AmigaDosInfo {
            filesystem_type: boot_block.filesystem_type,
            cache_mode: boot_block.cache_mode,
            international_mode: boot_block.international_mode,
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
    filesystem_cache_mode: CacheMode,
    filesystem_intl_mode: InternationalMode,
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
        BootBlockWriter::default()
            .width_filesystem_type(self.filesystem_type)
            .with_cache_mode(self.filesystem_cache_mode)
            .with_international_mode(self.filesystem_intl_mode)
            .write(&mut disk)?;

        RootBlock::with_volume_name(volume_name).write(&mut disk)?;

        Ok(AmigaDos {
            disk
        })
    }
}
