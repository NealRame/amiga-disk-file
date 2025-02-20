use std::cell::RefCell;
use std::path::{
    Path,
    PathBuf,
};
use std::rc::Rc;

use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
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
    file_type: FileType,
    name: String,
    path: PathBuf,
}

impl DirEntry {
    fn create(
        disk: Rc<RefCell<Disk>>,
        parent_path: &Path,
        addr: LBAAddress,
    ) -> Result<Self, Error> {
        let br = Block::new(disk, addr);

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

fn non_zero(addr: &u32) -> bool {
    *addr != 0
}

fn read_dir(
    disk: Rc<RefCell<Disk>>,
    block_addr: LBAAddress,
) -> Result<Vec<LBAAddress>, Error> {
    let hash_table = Block::new(disk.clone(), block_addr).read_hash_table()?;

    let mut entries = vec![];
    for mut block_addr in hash_table.iter().copied().filter(non_zero) {
        while block_addr != 0 {
            entries.push(block_addr as LBAAddress);
            block_addr = Block::new(
                disk.clone(),
                block_addr as LBAAddress,
            ).read_u32(BLOCK_HASH_CHAIN_NEXT_OFFSET)?;
        }
    }
    Ok(entries)
}

pub struct ReadDir {
    fs: Rc<RefCell<AmigaDosInner>>,
    entry_block_addr_list: Vec<LBAAddress>,
    path: PathBuf,
}

impl ReadDir {
    fn create<P: AsRef<Path>>(
        fs: Rc<RefCell<AmigaDosInner>>,
        path: P,
    ) -> Result<Self, Error> {
        let block_addr = fs.borrow().lookup(&path)?;
        let entry_block_addr_list = read_dir(
            fs.borrow().disk(),
            block_addr
        )?;

        Ok(Self {
            fs,
            entry_block_addr_list,
            path: PathBuf::from(path.as_ref()),
        })
    }
}

impl Iterator for ReadDir {
    type Item = Result<DirEntry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let fs = self.fs.borrow();
        let disk = fs.disk();
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
        ReadDir::create(self.inner.clone(), &path)
    }
}
