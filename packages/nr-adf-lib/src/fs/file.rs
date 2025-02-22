use std::cell::RefCell;
use std::ops;
use std::path::Path;
use std::rc::Rc;

use crate::block::*;
use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::block_type::*;
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

#[derive(Clone, Copy, Default)]
pub(super) struct FileDataBlockListEntry {
    // Address of the data block
    pub(super) data_block_address: usize,
    // Index of the data block address in the extension block
    pub(super) extension_block_index: usize,
    // Address of the extension block
    pub(super) extension_block_addr: LBAAddress,
}

impl FileDataBlockListEntry {
    pub(super) fn try_create(
        disk: Rc<RefCell<Disk>>,
        extension_block_addr: usize,
        extension_block_index: usize,
    ) -> Result<Option<Self>, Error> {
        let data_block_address = Block::new(
            disk,
            extension_block_addr,
        ).read_data_block_addr(
            BLOCK_DATA_LIST_SIZE - extension_block_index - 1
        )?;

        if data_block_address > 0 {
            Ok(Some(Self {
                data_block_address,
                extension_block_addr,
                extension_block_index,
            }))
        } else {
            Ok(None)
        }
    }
}

pub struct File {
    pub(super) fs: Rc<RefCell<AmigaDosInner>>,
    pub(super) block_data_list: Vec<FileDataBlockListEntry>,
    pub(super) block_data_offset: usize,
    pub(super) block_data_size: usize,
    pub(super) header_block_addr: LBAAddress,
    pub(super) mode: usize,
    pub(super) pos: usize,
    pub(super) size: usize,
}

impl File {
    fn release_extension_block(
        &mut self,
        entry: &FileDataBlockListEntry,
    ) -> Result<(), Error> {
        if let Some(last_entry) = self.block_data_list.last() {
            self.fs.borrow_mut().free_block(entry.extension_block_addr)?;

            let mut ext_block = Block::new(
                self.fs.borrow().disk(),
                last_entry.extension_block_addr,
            );

            ext_block.write_data_extension_block_addr(0)?;
            ext_block.write_checksum(BLOCK_CHECKSUM_OFFSET)?;
        }
        Ok(())
    }

    fn release_data_block(
        &mut self,
        entry: &FileDataBlockListEntry,
    ) -> Result<(), Error> {
        let mut ext_block = Block::new(
            self.fs.borrow().disk(),
            entry.extension_block_addr,
        );

        ext_block.write_data_block_addr(entry.extension_block_index, 0)?;
        ext_block.write_u32(
            BLOCK_DATA_LIST_HIGH_SEQ_OFFSET,
            entry.extension_block_index as u32,
        )?;

        self.fs.borrow_mut().free_block(entry.data_block_address)?;

        if entry.extension_block_index == 0 {
            self.release_extension_block(entry)?;
        } else {
            ext_block.write_checksum(BLOCK_CHECKSUM_OFFSET)?;
        }

        self.size -= self.size%self.block_data_size;
        self.pos = self.pos.min(self.size);

        Ok(())
    }

    fn alloc_extension_block(
        &mut self,
        entry: &FileDataBlockListEntry,
    ) -> Result<LBAAddress, Error> {
        let disk = self.fs.borrow().disk();
        let extension_block_addr = self.fs.borrow_mut().reserve_block()?;

        // Update the last file extension block with the address of the newly
        // allocated file extension block address.
        let mut last_ext_block = Block::new(disk.clone(), entry.data_block_address);

        last_ext_block.write_data_extension_block_addr(extension_block_addr)?;
        last_ext_block.write_checksum(BLOCK_CHECKSUM_OFFSET)?;

        // Initialize the newly allocated file extension block.
        let mut next_ext_block = Block::new(disk.clone(), extension_block_addr);

        next_ext_block.clear()?;
        next_ext_block.write_block_primary_type(BlockPrimaryType::List)?;
        next_ext_block.write_block_secondary_type(BlockSecondaryType::File)?;
        next_ext_block.write_u32(
            BLOCK_DATA_LIST_HEADER_KEY_OFFSET,
            extension_block_addr as u32,
        )?;
        next_ext_block.write_u32(
            BLOCK_DATA_LIST_PARENT_OFSET,
            self.header_block_addr as u32,
        )?;

        Ok(extension_block_addr)
    }

    fn alloc_data_block(
        &mut self,
        entry: &FileDataBlockListEntry,
    ) -> Result<FileDataBlockListEntry, Error> {
        let data_block_address = self.fs.borrow_mut().reserve_block()?;
        let (
            extension_block_addr,
            extension_block_index,
        ) = if entry.extension_block_index < BLOCK_DATA_LIST_SIZE {
            (entry.extension_block_addr, entry.extension_block_index + 1)
        } else {
            (self.alloc_extension_block(entry)?, 0)
        };

        let mut ext_block = Block::new(
            self.fs.borrow().disk(),
            extension_block_addr,
        );

        ext_block.write_data_block_addr(extension_block_index, data_block_address)?;
        ext_block.write_u32(
            BLOCK_DATA_LIST_PARENT_OFSET,
            (extension_block_index + 1) as u32,
        )?;

        let entry = FileDataBlockListEntry {
            data_block_address,
            extension_block_addr,
            extension_block_index,
        };

        self.block_data_list.push(entry);

        Ok(entry)
    }

    fn alloc_first_data_block(
        &mut self,
    ) -> Result<FileDataBlockListEntry, Error> {
        let data_block_address = self.fs.borrow_mut().reserve_block()?;
        let extension_block_addr = self.header_block_addr;
        let extension_block_index = 0;

        let mut ext_block = Block::new(
            self.fs.borrow().disk(),
            self.header_block_addr,
        );

        ext_block.write_data_block_addr(0, data_block_address)?;
        ext_block.write_u32(BLOCK_DATA_LIST_PARENT_OFSET, 1u32)?;

        let entry = FileDataBlockListEntry {
            data_block_address,
            extension_block_addr,
            extension_block_index,
        };

        self.block_data_list.push(entry);

        Ok(entry)
    }

    pub(super) fn pop_data_block_list_entry(
        &mut self,
    ) -> Result<(), Error> {
        if let Some(entry) = self.block_data_list.pop() {
            self.release_data_block(&entry)?
        }
        Ok(())
    }

    pub(super) fn push_data_block_list_entry(
        &mut self,
    ) -> Result<FileDataBlockListEntry, Error> {
        match self.block_data_list.last().copied() {
            Some(entry) => {
                self.alloc_data_block(&entry)
            },
            None => {
                self.alloc_first_data_block()
            },
        }
    }

    pub(super) fn get_data_block_list_entry(
        &self,
        pos: usize,
    ) -> Option<FileDataBlockListEntry> {
        self.block_data_list.get(pos/self.block_data_size).copied()
    }
}

impl AmigaDosInner {
    fn try_get_block_data_list(
        &self,
        header_block_addr: LBAAddress,
    ) -> Result<Vec<FileDataBlockListEntry>, Error> {
        let mut entries = Vec::new();
        let mut extension_block_address = header_block_addr;

        while extension_block_address != 0 {
            for extension_block_index in 0..BLOCK_DATA_LIST_SIZE {
                match FileDataBlockListEntry::try_create(
                    self.disk(),
                    extension_block_address,
                    extension_block_index,
                )? {
                    Some(entry) => entries.push(entry),
                    None => break,
                };
            }

            extension_block_address = Block::new(
                self.disk(),
                extension_block_address,
            ).read_data_extension_block_addr()?;
        }

        Ok(entries)
    }
}

impl AmigaDosInner {
    fn read_file_size(
        &self,
        header_block_addr: LBAAddress,
    ) -> Result<usize, Error> {
        Block::new(
            self.disk(),
            header_block_addr,
        ).read_file_size()
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

        let header_block_addr = fs.borrow().lookup(path)?;
        let size = fs.borrow().read_file_size(header_block_addr)?;
        let pos = if mode & FileMode::Append {
            size
        } else {
            0
        };

        let block_data_list = fs.borrow().try_get_block_data_list(header_block_addr)?;

        Ok(File {
            fs,
            block_data_list,
            block_data_offset,
            block_data_size,
            header_block_addr,
            mode,
            pos,
            size,
        })
    }
}
