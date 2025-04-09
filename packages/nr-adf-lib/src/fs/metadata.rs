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
use super::constants::*;


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

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            FileType::Dir  => 'd',
            FileType::File => ' ',
            FileType::Link => 'l',
        })
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Permissions (u32);

impl Permissions {
    pub fn owner_deletable(&self) -> bool  { self.0 & 0x00000001 == 0 }
    pub fn owner_executable(&self) -> bool { self.0 & 0x00000002 == 0 }
    pub fn owner_writable(&self) -> bool   { self.0 & 0x00000004 == 0 }
    pub fn owner_readable(&self) -> bool   { self.0 & 0x00000008 == 0 }

    pub fn group_deletable(&self) -> bool  { self.0 & 0x00000100 != 0 }
    pub fn group_executable(&self) -> bool { self.0 & 0x00000200 != 0 }
    pub fn group_writable(&self) -> bool   { self.0 & 0x00000400 != 0 }
    pub fn group_readable(&self) -> bool   { self.0 & 0x00000800 != 0 }

    pub fn other_deletable(&self) -> bool  { self.0 & 0x00001000 != 0 }
    pub fn other_executable(&self) -> bool { self.0 & 0x00002000 != 0 }
    pub fn other_writable(&self) -> bool   { self.0 & 0x00004000 != 0 }
    pub fn other_readable(&self) -> bool   { self.0 & 0x00008000 != 0 }
}

impl fmt::Display for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}{}{}{}{}{}{}{}{}{}",
            if self.owner_readable()   { 'r' } else { '-' },
            if self.owner_writable()   { 'w' } else { '-' },
            if self.owner_executable() { 'e' } else { '-' },
            if self.owner_deletable()  { 'd' } else { '-' },
            if self.group_readable()   { 'r' } else { '-' },
            if self.group_writable()   { 'w' } else { '-' },
            if self.group_executable() { 'e' } else { '-' },
            if self.group_deletable()  { 'd' } else { '-' },
            if self.other_readable()   { 'r' } else { '-' },
            if self.other_writable()   { 'w' } else { '-' },
            if self.other_executable() { 'e' } else { '-' },
            if self.other_deletable()  { 'd' } else { '-' },
        )
    }
}

#[derive(Clone, Debug)]
pub struct Metadata {
    header_block_address: LBAAddress,
    file_type: FileType,
    file_size: usize,
    permissions: Permissions,
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

        let permissions = Permissions(block.read_u32(BLOCK_PROTECT_OFFSET)?);

        let file_size = if file_type == FileType::File {
            block.read_file_size()?
        } else {
            0
        };

        Ok(Metadata {
            file_size,
            file_type,
            permissions,
            header_block_address,
            alteration_date,
            name,
        })
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let date = DateTime::<Local>::from(self.alteration_date).to_rfc3339();
        let size = if FileType::File == self.file_type {
            format!("{:>10}", self.file_size)
        } else {
            format!("{:>10}", "-")
        };

        write!(f, "{} {} {size} {date} {}",
            self.file_type,
            self.permissions,
            self.name,
        )
    }
}

impl Metadata {
    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    pub fn permissions(&self) -> Permissions {
        self.permissions
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
