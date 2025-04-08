use std::fmt;
use std::path::Path;
use std::time::SystemTime;

use chrono::{
    DateTime,
    Local,
};

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

impl From<FileType> for BlockSecondaryType {
    fn from(value: FileType) -> Self {
        match value {
            FileType::Dir  => Self::Directory,
            FileType::File => Self::File,
            FileType::Link => Self::SoftLink,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Metadata {
    header_block_address: LBAAddress,
    file_type: FileType,
    file_size: usize,
    alteration_date: SystemTime,
    name: String,
}

impl TryFrom<&Block> for Metadata {
    type Error = Error;

    fn try_from(block: &Block) -> Result<Self, Self::Error> {
        let header_block_address = block.address;
        let alteration_date = block.read_alteration_date()?;
        let name = block.read_name()?;

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

        let file_size = if file_type == FileType::File {
            block.read_file_size()?
        } else {
            0
        };

        Ok(Metadata {
            file_size,
            file_type,
            header_block_address,
            alteration_date,
            name,
        })
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size = if FileType::File == self.file_type {
            format!("{:>10}", self.file_size)
        } else {
            format!("{:>10}", "-")
        };

        let date = DateTime::<Local>::from(self.alteration_date).to_rfc3339();

        write!(f, "{size} {date} {}", self.name)
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

    pub fn size(&self) -> usize {
        self.file_size
    }

    pub fn header_block_address(&self) -> LBAAddress {
        self.header_block_address
    }

    pub fn alteration_date(&self) -> SystemTime {
        self.alteration_date
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl AmigaDosInner {
    pub(super) fn metadata(
        &self,
        header_block_address: LBAAddress,
    ) -> Result<Metadata, Error> {
        let disk = self.disk();
        let block = Block::new(disk, header_block_address);

        Metadata::try_from(&block)
    }
}

impl AmigaDos {
    /// Given a path, queries the file system to get information about a file,
    /// directory, etc.
    pub fn metadata<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Metadata, Error> {
        let header_block_address = self.lookup(path.as_ref())?;
        self.inner.borrow().metadata(header_block_address)
    }

    /// Returns Ok(true) if the path points at an existing entity.
    pub fn exists<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<bool, Error> {
        match self.metadata(path) {
            Err(Error::NotFoundError) => Ok(false),
            Err(err) => Err(err),
            _ => Ok(true),
        }
    }

}
