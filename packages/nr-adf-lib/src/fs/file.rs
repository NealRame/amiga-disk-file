use std::cell::RefCell;
use std::ffi::OsStr;
use std::ops;
use std::path::Path;
use std::rc::Rc;

use crate::block::*;
use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::amiga_dos_options::*;
use super::block_type::*;
use super::constants::*;
use super::dir::*;
use super::file_open::*;
use super::metadata::*;


#[repr(usize)]
#[derive(Clone, Debug)]
pub enum FileMode {
    /// This mode means that the file should be read-able whrn opened.
    Read     = 0x01,

    /// This mode means that the file should be write-able when opened.
    /// If the file already exists, any write calls on it will overwrite its
    /// contents.
    Write    = 0x02,
}

type FileModeMask = usize;

impl Default for FileMode {
    fn default() -> Self {
        Self::Read
    }
}

impl ops::Not for FileMode {
    type Output = FileModeMask;

    fn not(self) -> Self::Output {
        !(self as FileModeMask)
    }
}

impl ops::BitAnd<FileModeMask> for FileMode {
    type Output = FileModeMask;

    fn bitand(self, rhs: FileModeMask) -> Self::Output {
        rhs & self as FileModeMask
    }
}

impl ops::BitAnd<FileMode> for FileModeMask {
    type Output = FileModeMask;

    fn bitand(self, rhs: FileMode) -> Self::Output {
        rhs & self
    }
}

impl ops::BitOr<FileMode> for FileMode {
    type Output = FileModeMask;

    fn bitor(self, rhs: FileMode) -> Self::Output {
        self as FileModeMask | rhs as FileModeMask
    }
}

impl ops::BitOr<FileModeMask> for FileMode {
    type Output = FileModeMask;

    fn bitor(self, rhs: FileModeMask) -> Self::Output {
        rhs | self as FileModeMask
    }
}

impl ops::BitOr<FileMode> for FileModeMask {
    type Output = FileModeMask;

    fn bitor(self, rhs: FileMode) -> Self::Output {
        rhs | self
    }
}


pub(super) fn check_file_mode(
    mode: FileMode,
    mode_mask: FileModeMask,
) -> Result<(), Error> {
    if (mode_mask & mode as usize) == 0 {
        Err(Error::BadFileDescriptor)
    } else {
        Ok(())
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
        let index = BLOCK_DATA_LIST_SIZE - extension_block_index - 1;
        let data_block_address = Block::new(
            disk,
            extension_block_addr,
        ).read_block_table_address(index)?;

        if let Some(data_block_address) = data_block_address {
            Ok(Some(Self {
                data_block_address,
                extension_block_addr,
                extension_block_index,
            }))
        } else {
            Ok(None)
        }
    }

    pub(super) fn try_get_block_data_list(
        disk: Rc<RefCell<Disk>>,
        header_block_addr: LBAAddress,
    ) -> Result<Vec<FileDataBlockListEntry>, Error> {
        let mut entries = Vec::new();
        let mut block_address = Some(header_block_addr);

        while let Some(extension_block_address) = block_address {
            for extension_block_index in 0..BLOCK_DATA_LIST_SIZE {
                match FileDataBlockListEntry::try_create(
                    disk.clone(),
                    extension_block_address,
                    extension_block_index,
                )? {
                    Some(entry) => entries.push(entry),
                    None => break,
                };
            }

            block_address = Block::new(
                disk.clone(),
                extension_block_address,
            ).read_data_list_extension_address()?;
        }

        Ok(entries)
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

            ext_block.write_data_list_extension_address(0)?;
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

        ext_block.write_block_table_address(entry.extension_block_index, 0)?;
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

        last_ext_block.write_data_list_extension_address(extension_block_addr)?;
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
            BLOCK_DATA_LIST_PARENT_OFFSET,
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

        ext_block.write_block_table_address(extension_block_index, data_block_address)?;
        ext_block.write_u32(
            BLOCK_DATA_LIST_PARENT_OFFSET,
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

        ext_block.write_block_table_address(0, data_block_address)?;
        ext_block.write_u32(BLOCK_DATA_LIST_PARENT_OFFSET, 1u32)?;

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

impl File {
    pub(super) fn try_open(
        fs: &AmigaDos,
        path: &Path,
        mode: usize,
    ) -> Result<Self, Error> {
        let filesystem_type = fs.get_filesystem_type()?;
        let metadata = fs.metadata(path)?;

        let header_block_addr = metadata.header_block_address();
        let size = metadata.size();
        let pos = 0;

        let block_data_list = FileDataBlockListEntry::try_get_block_data_list(
            fs.disk(),
            header_block_addr,
        )?;

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

        Ok(File {
            fs: fs.inner.clone(),
            block_data_list,
            block_data_offset,
            block_data_size,
            header_block_addr,
            mode,
            pos,
            size,
        })
    }

    pub(super) fn try_create(
        fs: &AmigaDos,
        path: &Path,
        mode: usize,
        create_new: bool,
    ) -> Result<File, Error> {
        let parent_path = path.parent().ok_or(Error::InvalidPathError)?;
        let mut parent_dir = Dir::try_with_path(fs, parent_path)?;

        let file_name
            = path
                .file_name()
                .and_then(OsStr::to_str)
                .ok_or(Error::InvalidPathError)?;

        let (_, created) = parent_dir.create_entry(file_name, FileType::File)?;

        if created {
            return File::try_open(fs, path, mode);
        }

        if create_new {
            return Err(Error::AlreadyExists);
        }

        let mut file = File::try_open(fs, path, mode)?;

        file.set_len(0)?;
        Ok(file)
    }
}

impl File {
    pub fn options() -> OpenOptions {
        OpenOptions::default()
    }
}
