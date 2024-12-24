use std::path::{
    Path,
    PathBuf,
};

use crate::disk::*;
use crate::errors::*;

use super::block::*;
use super::block_type::*;


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

        br.verify_block_primary_type(BlockPrimaryType::Header)?;

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
    pub fn try_from_disk<P: AsRef<Path>>(
        disk: &'disk Disk,
        block_addr: LBAAddress,
        path: P,
    ) -> Result<Self, Error> {
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
