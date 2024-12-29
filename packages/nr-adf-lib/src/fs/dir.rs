use std::path::{
    Path,
    PathBuf,
};

use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::block::*;
use super::block_type::*;
use super::boot_block::*;


fn path_split<P: AsRef<Path>>(
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

fn path_lookup<P: AsRef<Path>>(
    disk: &Disk,
    path: P,
) -> Result<LBAAddress, Error> {
    if let Some(path) = path_split(path) {
        let boot_block = BootBlock::try_from_disk(disk)?;
        let international_mode = boot_block.get_international_mode();
        let mut block_addr = boot_block.get_root_block_address();

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

#[derive(Clone, Copy, Debug)]
pub enum FileType {
    File,
    Dir,
    Link,
}

impl Default for FileType {
    fn default() -> Self { Self::File }
}

#[derive(Clone, Debug, Default)]
pub struct DirEntry {
    file_type: FileType,
    name: String,
    path: PathBuf,
}

impl DirEntry {
    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }
}

impl DirEntry {
    pub fn try_from_disk(
        parent_path: &Path,
        disk: &Disk,
        addr: LBAAddress,
    ) -> Result<Self, Error> {
        let br = BlockReader::try_from_disk(disk, addr)?;

        br.check_block_primary_type(BlockPrimaryType::Header)?;

        let file_type = match br.read_block_secondary_type()? {
            BlockSecondaryType::File |
            BlockSecondaryType::HardLinkFile => {
                FileType::File
            },
            BlockSecondaryType::Root |
            BlockSecondaryType::Directory |
            BlockSecondaryType::HardLinkDirectory => {
                FileType::Dir
            },
            BlockSecondaryType::SoftLink => {
                FileType::Link
            },
        };

        let name = br.read_name()?;
        let path = parent_path.join(&name);

        Ok(Self {
            file_type,
            name,
            path,
        })
    }
}

pub struct ReadDir<'disk> {
    disk: &'disk Disk,
    entry_block_addr_list: Vec<LBAAddress>,
    path: PathBuf,
}

impl<'disk> ReadDir<'disk> {
    fn try_from_disk<P: AsRef<Path>>(
        disk: &'disk Disk,
        path: P,
    ) -> Result<Self, Error> {
        let block_addr = path_lookup(disk, &path)?;
        let br = BlockReader::try_from_disk(disk, block_addr)?;
        let entry_block_addr_list = br.read_dir()?;

        Ok(Self {
            disk,
            entry_block_addr_list,
            path: PathBuf::from(path.as_ref()),
        })
    }
}

impl Iterator for ReadDir<'_> {
    type Item = Result<DirEntry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.entry_block_addr_list
            .pop()
            .map(|addr| DirEntry::try_from_disk(&self.path, self.disk, addr))
    }
}

impl AmigaDos {
    pub fn read_dir<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<ReadDir, Error> {
        ReadDir::try_from_disk(self.disk(), &path)
    }
}
