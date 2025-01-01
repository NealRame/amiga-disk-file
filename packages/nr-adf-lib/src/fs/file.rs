use std::ops;
use std::path::Path;

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
    Append   = 0x4,

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

pub struct File<'disk> {
    disk: &'disk Disk,
    block_data_offset: usize,
    block_data_size: usize,
    block_data_list_max_size: usize,
    header_block_addr: LBAAddress,
    mode: usize,
    pos: usize,
    size: usize,
}

impl<> File<'_> {
    fn get_data_block_addr(&self) -> Result<usize, Error> {
        let mut addr = self.header_block_addr;
        let mut pos = self.pos;

        // TODO: it feels like doing this every time is not very efficient.
        // We'll try to optimize that later.
        while pos >= self.block_data_list_max_size {
            addr = BlockReader::try_from_disk(
                self.disk,
                addr,
            )?.read_data_extension_block_addr()?;
            pos -= self.block_data_list_max_size;
        }

        // let block_index = pos/self.block_data_size;
        let block_index = BLOCK_BLOCK_DATA_LIST_SIZE - 1 - pos/self.block_data_size;

        BlockReader::try_from_disk(
            self.disk,
            addr,
        )?.read_data_block_addr(block_index)
    }

    fn read_data(
        &mut self,
        buf: &mut [u8],
        block_data_pos: usize,
    ) -> Result<(), Error> {
        let block_addr = self.get_data_block_addr()?;
        let block = BlockReader::try_from_disk(self.disk, block_addr)?;

        block.read_u8_array(self.block_data_offset + block_data_pos, buf)
    }
}

impl<> File<'_> {
    pub fn read(
        &mut self,
        mut buf: &mut [u8],
    ) -> Result<usize, Error> {
        if ! self.mode & FileMode::Read {
            return Err(Error::BadFileDescriptor);
        }

        if self.pos >= self.size {
            return Ok(0);
        }

        let total = buf.len();
        let mut count = 0;

        while count < total && self.pos < self.size {
            let read_data_pos = self.pos%self.block_data_size;
            let read_data_len =
                buf.len()
                    .min(self.size - self.pos)
                    .min(self.block_data_size - read_data_pos);

            self.read_data(&mut buf[..read_data_len], read_data_pos)?;
            self.pos += read_data_len;

            count += read_data_len;
            buf = &mut buf[read_data_len..];
        }

        Ok(count)
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

        let disk = self.disk();
        let filesystem_type = self.get_filesystem_type()?;
        let header_block_addr = self.lookup(path)?;
        let header_block = BlockReader::try_from_disk(disk, header_block_addr)?;

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

        let size = header_block.read_file_size()?;
        let pos = if mode & FileMode::Append {
            size
        } else {
            0
        };

        Ok(File {
            disk,
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
