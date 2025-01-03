use std::path::{
    Path,
    PathBuf,
};

use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
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
    fn create(
        disk: &Disk,
        parent_path: &Path,
        addr: LBAAddress,
    ) -> Result<Self, Error> {
        let br = BlockReader::try_from_disk(disk, addr)?;

        br.check_block_primary_type(&[BlockPrimaryType::Header])?;

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

pub struct ReadDir<'fs> {
    fs: &'fs AmigaDos,
    entry_block_addr_list: Vec<LBAAddress>,
    path: PathBuf,
}

impl<'fs> ReadDir<'fs> {
    fn create<P: AsRef<Path>>(
        fs: &'fs AmigaDos,
        path: P,
    ) -> Result<Self, Error> {
        let block_addr = fs.lookup(&path)?;
        let br = BlockReader::try_from_disk(fs.disk(), block_addr)?;
        let entry_block_addr_list = br.read_dir()?;

        Ok(Self {
            fs,
            entry_block_addr_list,
            path: PathBuf::from(path.as_ref()),
        })
    }
}

impl Iterator for ReadDir<'_> {
    type Item = Result<DirEntry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let disk = self.fs.disk();
        let path = &self.path;

        self.entry_block_addr_list
            .pop()
            .map(|addr| DirEntry::create(disk, path, addr))
    }
}

impl AmigaDos {
    pub fn read_dir<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<ReadDir, Error> {
        ReadDir::create(self, &path)
    }
}
