use std::cell::RefCell;
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};
use std::rc::Rc;

use crate::block::*;
use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::block_type::*;
use super::boot_block::*;
use super::constants::*;
use super::metadata::*;
use super::name::*;
use super::path_split::*;

fn check_directory(
    disk: Rc<RefCell<Disk>>,
    addr: LBAAddress,
) -> Result<(), Error> {
    let block = Block::new(disk.clone(), addr);

    match block.read_block_secondary_type()? {
        BlockSecondaryType::Directory |
        BlockSecondaryType::Root => {
            Ok(())
        }
        _ => {
            Err(Error::NotADirectoryError)
        }
    }
}

#[derive(Clone, Debug)]
pub struct DirEntry {
    metadata: Metadata,
    name: String,
    path: PathBuf,
}

impl DirEntry {
    pub fn file_type(&self) -> FileType {
        self.metadata.file_type()
    }

    pub fn metadata(&self) -> Metadata {
        self.metadata
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }
}

#[derive(Clone, Debug)]
pub(super) struct Dir {
    fs: Rc<RefCell<AmigaDosInner>>,
    pub(super) header_block_address: LBAAddress,
    pub(super) path: PathBuf,
}

impl Dir {
    pub(super) fn try_with_block_address<P: AsRef<Path>>(
        fs: &AmigaDos,
        header_block_address: LBAAddress,
        path: P,
    ) -> Result<Self, Error> {
        let block = Block::new(fs.disk(), header_block_address);

        block.check_block_primary_type(&[BlockPrimaryType::Header])?;
        block.check_block_secondary_type(&[
            BlockSecondaryType::Directory,
            BlockSecondaryType::Root,
        ])?;

        Ok(Self {
            fs: fs.inner.clone(),
            header_block_address,
            path: PathBuf::from(path.as_ref()),
        })
    }

    pub(super) fn try_with_path<P: AsRef<Path>>(
        fs: &AmigaDos,
        path: P,
    ) -> Result<Self, Error> {
        let block_addr = fs.lookup(path.as_ref())?;

        Self::try_with_block_address(
            fs,
            block_addr,
            path.as_ref(),
        )
    }
}

fn find_in_hash_chain(
    disk: Rc<RefCell<Disk>>,
    name: &str,
    mut addr: Option<LBAAddress>,
) -> Result<Option<LBAAddress>, Error> {
    while let Some(block_addr) = addr {
        let block = Block::new(disk.clone(), block_addr);
        let entry_name = block.read_name()?;

        if entry_name == name {
            return Ok(Some(block_addr));
        }
        addr = AmigaDos::to_address(block.read_u32(BLOCK_HASH_CHAIN_NEXT_OFFSET)?);
    }

    Ok(None)
}

impl Dir {
    pub(super) fn lookup(
        &self,
        name: &str,
    ) -> Result<Option<LBAAddress>, Error> {
        let disk = self.fs.borrow().disk();
        let boot_block = BootBlockReader::try_from_disk(disk.clone())?;
        let international_mode = boot_block.get_international_mode();

        let hash_index = hash_name(name, international_mode);
        let head = Block::new(
            disk.clone(),
            self.header_block_address,
        ).read_block_table_address(hash_index)?;

        find_in_hash_chain(disk.clone(), name, head)
    }

    pub(super) fn create_entry(
        &mut self,
        name: &str,
        file_type: FileType,
    ) -> Result<(LBAAddress, bool), Error> {
        let disk = self.fs.borrow().disk();

        let parent_block_addr = self.header_block_address;

        let boot_block = BootBlockReader::try_from_disk(disk.clone())?;
        let international_mode = boot_block.get_international_mode();

        let hash_index = hash_name(name, international_mode);
        let head = Block::new(
            disk.clone(),
            parent_block_addr,
        ).read_block_table_address(hash_index)?;

        if let Some(addr) = find_in_hash_chain(disk.clone(), name, head)? {
            check_directory(disk.clone(), addr)?;

            Ok((addr, false))
        } else {
            let addr = self.fs.borrow_mut().reserve_block()?;

            Block::new(
                disk.clone(),
                addr,
            ).init_header(file_type.into(), name, parent_block_addr, head)?;

            Block::new(
                disk.clone(),
                parent_block_addr,
            ).write_block_table_address(hash_index, addr)?;

            Ok((addr, true))
        }
    }
}

#[derive(Clone, Debug)]
pub struct DirIterator {
    current_table_index: usize,
    current_table_addr: Option<LBAAddress>,
    disk: Rc<RefCell<Disk>>,
    header_block_address: LBAAddress,
    path: PathBuf,
}

impl DirIterator {
    fn block_table_next(
        &mut self
    ) -> Result<(), Error> {
        let disk = self.disk.clone();
        let header_block = Block::new(disk, self.header_block_address);

        while self.current_table_addr.is_none()
            && self.current_table_index < BLOCK_TABLE_SIZE {
            self.current_table_addr = header_block.read_block_table_address(self.current_table_index)?;
            self.current_table_index += 1;
        }

        Ok(())
    }

    fn block_chain_next(
        &mut self,
        block: &Block,
    ) -> Result<(), Error> {
        self.current_table_addr = match block.read_hash_chain_next_address() {
            Ok(addr) => addr,
            Err(err) => {
                return Err(err);
            }
        };
        Ok(())
    }
}

impl Iterator for DirIterator {
    type Item = Result<DirEntry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Err(err) = self.block_table_next() {
            return Some(Err(err));
        }

        if self.current_table_addr.is_none() {
            return None;
        }

        let block = Block::new(
            self.disk.clone(),
            self.current_table_addr.unwrap(),
        );

        let metadata = match Metadata::try_from(&block) {
            Ok(metadata) => metadata,
            Err(err) => {
                return Some(Err(err));
            }
        };

        let name = match block.read_name() {
            Ok(name) => name,
            Err(err) => {
                return Some(Err(err));
            }
        };

        let path = self.path.join(&name);

        if let Err(err) = self.block_chain_next(&block) {
            return Some(Err(err));
        }

        Some(Ok(DirEntry { metadata, name, path }))
    }
}

impl AmigaDos {
    /// Returns an iterator over the entries within a directory.
    pub fn read_dir<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<DirIterator, Error> {
        let dir = Dir::try_with_path(self, path)?;

        Ok(DirIterator {
            current_table_index: 0,
            current_table_addr: None,
            disk: self.disk(),
            header_block_address: dir.header_block_address,
            path: dir.path.clone(),
        })
    }

    /// Creates a new, empty directory at the provided path.
    /// Errors:
    /// - When a parent of the given path doesn’t exist. Use `create_dir_all`
    ///   function to create a directory and all its missing parents at the
    ///   same time.
    pub fn create_dir<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<(), Error> {
        let path = path.as_ref();
        let parent_path = path.parent().ok_or(Error::InvalidPathError)?;
        let mut parent_dir = Dir::try_with_path(self, parent_path)?;

        let dir_name
            = path
                .file_name()
                .and_then(OsStr::to_str)
                .ok_or(Error::InvalidPathError)?;

        parent_dir.create_entry(dir_name, FileType::Dir)?;

        Ok(())
    }

    /// Create a directory and all of its parent components if they are missing.
    pub fn create_dir_all<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<(), Error> {
        if let Some(path) = path_split(path) {
            let disk = self.inner.borrow().disk();

            let boot_block = BootBlockReader::try_from_disk(disk.clone())?;

            let mut dir = Dir::try_with_block_address(
                self,
                boot_block.get_root_block_address(),
                PathBuf::default(),
            )?;

            for name in path {
                let (addr, _) = dir.create_entry(&name, FileType::Dir)?;

                dir = Dir::try_with_block_address(
                    self,
                    addr,
                    PathBuf::default(),
                )?;
            }

            Ok(())
        } else {
            Err(Error::InvalidPathError)
        }
    }
}
