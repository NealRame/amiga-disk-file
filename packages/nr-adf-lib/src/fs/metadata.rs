use std::path::Path;
use std::time::SystemTime;

use crate::block::*;
use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::block_type::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileType {
    File,
    Dir,
    Link,
}

impl Default for FileType {
    fn default() -> Self { Self::File }
}

#[derive(Clone, Copy, Debug)]
pub struct Metadata {
    header_block_address: LBAAddress,
    file_type: FileType,
    len: usize,
    alteration_date: SystemTime,
}

impl TryFrom<&Block> for Metadata {
    type Error = Error;

    fn try_from(block: &Block) -> Result<Self, Self::Error> {
        let header_block_address = block.address;
        let len = block.read_file_size()?;
        let alteration_date = block.read_alteration_date()?;

        let file_type = match block.read_block_secondary_type()? {
            BlockSecondaryType::Root |
            BlockSecondaryType::Directory |
            BlockSecondaryType::HardLinkDirectory => {
                FileType::Dir
            },
            BlockSecondaryType::File |
            BlockSecondaryType::HardLinkFile => {
                FileType::File
            },
            BlockSecondaryType::SoftLink => {
                FileType::Link
            },
        };

        Ok(Metadata {
            file_type,
            header_block_address,
            len,
            alteration_date,
        })
    }
}

impl Metadata {
    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    pub fn is_dir(&self) -> bool {
        self.file_type == FileType::Dir
    }

    pub fn is_file(&self) -> bool {
        self.file_type == FileType::File
    }

    pub fn is_symlink(&self) -> bool {
        self.file_type == FileType::Link
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn header_block_address(&self) -> LBAAddress {
        self.header_block_address
    }

    pub fn alteration_date(&self) -> SystemTime {
        self.alteration_date
    }
}

impl AmigaDos {
    /// Given a path, queries the file system to get information about a file,
    /// directory, etc.
    pub fn metadata<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Metadata, Error> {
        let header_block_address = self.inner.borrow().lookup(path)?;
        let disk = self.inner.borrow().disk();
        let block = Block::new(disk, header_block_address);

        Metadata::try_from(&block)
    }
}
