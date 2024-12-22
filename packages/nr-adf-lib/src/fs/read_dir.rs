use std::path::{
    Path,
    PathBuf,
};

use crate::disk::*;
use crate::errors::*;

use super::block::*;
use super::block_type::*;
use super::constants::*;


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
    block_addr_hash_next: LBAAddress,
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
        disk: &Disk,
        block_addr: LBAAddress,
        parent_path: &Path,
    ) -> Result<Self, Error> {
        let br = BlockReader::try_from_disk(disk, block_addr)?;

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

        let block_addr_hash_next = br.read_u32(BLOCK_HASH_CHAIN_NEXT_OFFSET)? as usize;
        let name = br.read_name()?;
        let path = parent_path.join(&name);

        Ok(Self {
            block_addr_hash_next,
            file_type,
            name,
            path,
        })
    }

}

pub struct ReadDir<'disk> {
    disk: &'disk Disk,
    path: PathBuf,

    hash_table: Vec<u32>,

    next_hash_table_index: usize,
    next_block_addr: LBAAddress,
}

impl<'disk> ReadDir<'disk> {
    pub fn try_from_disk(
        disk: &'disk Disk,
        block_addr: LBAAddress,
        path: PathBuf,
    ) -> Result<Self, Error> {
        let br = BlockReader::try_from_disk(disk, block_addr)?;

        br.verify_block_primary_type(BlockPrimaryType::Header)?;
        br.verify_block_secondary_type(&[
            BlockSecondaryType::Root,
            BlockSecondaryType::Directory,
        ])?;

        let hash_table = br.read_u32_vector(
            BLOCK_HASH_TABLE_OFFSET,
            BLOCK_HASH_TABLE_SIZE,
        )?;

        Ok(Self {
            disk,
            path,
            hash_table,
            next_hash_table_index: 0,
            next_block_addr: 0,
        })
    }
}

impl Iterator for ReadDir<'_> {
    type Item = Result<DirEntry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_hash_table_index >= BLOCK_HASH_TABLE_SIZE {
            return None;
        }

        if self.next_block_addr == 0 {
            self.next_hash_table_index =
                self.hash_table
                    .iter()
                    .skip(self.next_hash_table_index)
                    .position(|&v| v != 0)
                    .unwrap_or(BLOCK_HASH_TABLE_SIZE);

            self.next_block_addr =
                self.hash_table
                    .get(self.next_hash_table_index)
                    .copied()
                    .unwrap_or(0) as LBAAddress;

            return self.next();
        }


        let entry = DirEntry::try_from_disk(
            self.disk,
            self.next_block_addr,
            self.path.as_ref(),
        );

        self.next_block_addr =
            entry.as_ref()
                .map(|entry| entry.block_addr_hash_next)
                .unwrap_or(0);

        Some(entry)
    }
}
