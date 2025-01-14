use std::cell::RefCell;
use std::ops;
use std::path::Path;
use std::rc::Rc;

use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::block::*;
use super::constants::*;
use super::options::*;


#[repr(usize)]
#[derive(Clone, Debug)]
pub enum FileMode {
    /// This mode means that the file should be read-able whrn opened.
    Read     = 0x01,

    /// This mode means that the file should be write-able when opened.
    /// If the file already exists, any write calls on it will overwrite its
    /// contents.
    Write    = 0x02,

    /// This mode means that the file should be write-able when opened.
    /// If the file already exists, any write calls on it will append data
    /// instead of overwriting previous contents.
    Append   = 0x04,

    /// This mode means that the file should be write-able when opened.
    /// If the file already exists, it will be truncated to 0 length.
    Truncate = 0x08,
}

impl Default for FileMode {
    fn default() -> Self {
        Self::Read
    }
}

impl ops::BitAnd<usize> for FileMode {
    type Output = bool;

    fn bitand(self, rhs: usize) -> Self::Output {
        (self as usize & rhs) != 0
    }
}

impl ops::BitAnd<FileMode> for usize {
    type Output = bool;

    fn bitand(self, rhs: FileMode) -> Self::Output {
        (self & rhs as usize) != 0
    }
}

impl ops::BitOr<FileMode> for FileMode {
    type Output = usize;

    fn bitor(self, rhs: FileMode) -> Self::Output {
        self as usize | rhs as usize
    }
}

impl ops::BitOr<usize> for FileMode {
    type Output = usize;

    fn bitor(self, rhs: usize) -> Self::Output {
        rhs | self as usize
    }
}

impl ops::BitOr<FileMode> for usize {
    type Output = usize;

    fn bitor(self, rhs: FileMode) -> Self::Output {
        rhs | self
    }
}

pub struct File {
    pub(super) fs: Rc<RefCell<AmigaDosInner>>,
    pub(super) block_data_offset: usize,
    pub(super) block_data_size: usize,
    pub(super) block_data_list_max_size: usize,
    pub(super) header_block_addr: LBAAddress,
    pub(super) mode: usize,
    pub(super) pos: usize,
    pub(super) size: usize,
}

impl File {
    fn get_current_data_block_index(&self) -> Result<(LBAAddress, usize), Error> {
        let fs = self.fs.borrow();
        let disk = fs.disk();

        let mut addr = self.header_block_addr;
        let mut pos = self.pos;

        // TODO: it feels like doing this every time is not very efficient.
        // We'll try to optimize that later.
        while pos >= self.block_data_list_max_size {
            addr = BlockReader::try_from_disk(
                disk,
                addr,
            )?.read_data_extension_block_addr()?;
            pos -= self.block_data_list_max_size;
        }

        let index = BLOCK_BLOCK_DATA_LIST_SIZE - 1 - pos/self.block_data_size;

        Ok((addr, index))
    }

    pub(super) fn get_data_block_addr(&self) -> Result<usize, Error> {
        let (addr, index) = self.get_current_data_block_index()?;

        BlockReader::try_from_disk(
            self.fs.borrow().disk(),
            addr,
        )?.read_data_block_addr(index)
    }

    pub(super) fn get_new_data_block_addr(
        &self,
    ) -> Result<LBAAddress, Error> {
        let (addr, index) = self.get_current_data_block_index()?;

        let mut fs = self.fs.borrow_mut();
        let new_block_addr = fs.reserve_block()?;

        BlockWriter::try_from_disk(
            fs.disk_mut(),
            addr
        )?.write_data_block_addr(index, new_block_addr).and(Ok(new_block_addr))
    }
}

impl AmigaDosInner {
    fn read_file_size(
        &self,
        file_header_block_addr: LBAAddress,
    ) -> Result<usize, Error> {
        BlockReader::try_from_disk(
            self.disk(),
            file_header_block_addr,
        )?.read_file_size()
    }
}

impl AmigaDos {
    /// Attempts to open a file.
    pub fn open<P: AsRef<Path>>(
        &self,
        path: P,
        mode: usize,
    ) -> Result<File, Error> {
        if mode & FileMode::Append
        && mode & FileMode::Truncate {
            return Err(Error::InvalidFileModeError);
        }

        let fs = self.inner.clone();

        let filesystem_type = fs.borrow().get_filesystem_type()?;

        let (
            block_data_offset,
            block_data_size,
        ) = match filesystem_type {
            FilesystemType::FFS => (
                BLOCK_DATA_FFS_OFFSET,
                BLOCK_DATA_FFS_SIZE,
            ),
            FilesystemType::OFS => (
                BLOCK_DATA_OFS_OFFSET,
                BLOCK_DATA_OFS_SIZE,
            ),
        };
        let block_data_list_max_size = block_data_size*BLOCK_BLOCK_DATA_LIST_SIZE;

        let header_block_addr = fs.borrow().lookup(path)?;
        let size = fs.borrow().read_file_size(header_block_addr)?;
        let pos = if mode & FileMode::Append {
            size
        } else {
            0
        };

        Ok(File {
            fs,
            block_data_offset,
            block_data_size,
            block_data_list_max_size,
            header_block_addr,
            mode,
            pos,
            size,
        })
    }
}
