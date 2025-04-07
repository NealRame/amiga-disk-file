use std::cell::RefCell;
use std::ops;
use std::path::Path;
use std::rc::Rc;
use std::time::SystemTime;

use crate::block::*;
use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::amiga_dos_options::*;
use super::block_type::*;
use super::constants::*;
use super::dir::*;
use super::file_open::*;
use super::path::*;


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

pub(super) fn test_file_mode(
    mode: FileMode,
    mode_mask: FileModeMask,
) -> bool {
    (mode_mask & mode) != 0
}

pub(super) fn check_file_mode(
    mode: FileMode,
    mode_mask: FileModeMask,
) -> Result<(), Error> {
    if !test_file_mode(mode, mode_mask) {
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
    pub(super) extension_block_address: LBAAddress,
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
                extension_block_address: extension_block_addr,
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

fn get_data_block_info(
    fs: &AmigaDos,
) -> Result<(usize, usize), Error> {
    Ok(match fs.get_filesystem_type()? {
        FilesystemType::FFS => (BLOCK_DATA_FFS_OFFSET, BLOCK_DATA_FFS_SIZE),
        FilesystemType::OFS => (BLOCK_DATA_OFS_OFFSET, BLOCK_DATA_OFS_SIZE),
    })
}

pub struct File {
    pub(super) fs: Rc<RefCell<AmigaDosInner>>,
    pub(super) block_data_list: Vec<FileDataBlockListEntry>,
    pub(super) block_data_offset: usize,
    pub(super) block_data_size: usize,
    pub(super) header_block_address: LBAAddress,
    pub(super) mode: usize,
    pub(super) pos: usize,
    pub(super) size: usize,
}

impl File {
    fn update_extension_block_next(
        &self,
        ext_block_addr: LBAAddress,
        next_ext_block_addr: LBAAddress,
    ) -> Result<(), Error> {
        let mut ext_block = Block::new(
            self.fs.borrow().disk(),
            ext_block_addr,
        );

        ext_block.write_u32(
            BLOCK_DATA_LIST_EXTENSION_OFFSET,
            next_ext_block_addr as u32,
        )?;
        ext_block.write_checksum()?;

        Ok(())
    }

    fn set_extension_block_data_block(
        &mut self,
        entry: FileDataBlockListEntry,
    ) -> Result<(), Error> {
        if entry.extension_block_index >= BLOCK_TABLE_SIZE {
            return Err(Error::InvalidDataBlockIndexError(entry.extension_block_index));
        }

        let mut ext_block = Block::new(
            self.fs.borrow().disk(),
            entry.extension_block_address,
        );

        ext_block.write_block_table_address(
            BLOCK_TABLE_SIZE - entry.extension_block_index - 1,
            entry.data_block_address,
        )?;
        ext_block.write_u32(
            BLOCK_DATA_LIST_HIGH_SEQ_OFFSET,
            entry.extension_block_index as u32 + 1,
        )?;
        ext_block.write_checksum()?;

        Ok(())
    }

    fn unset_extension_block_data_block(
        &mut self,
        entry: FileDataBlockListEntry,
    ) -> Result<(), Error> {
        if entry.extension_block_index >= BLOCK_TABLE_SIZE {
            return Err(Error::InvalidDataBlockIndexError(entry.extension_block_index));
        }

        let mut ext_block = Block::new(
            self.fs.borrow().disk(),
            entry.extension_block_address,
        );

        ext_block.write_block_table_address(
            BLOCK_TABLE_SIZE - entry.extension_block_index - 1,
            0,
        )?;
        ext_block.write_u32(
            BLOCK_DATA_LIST_HIGH_SEQ_OFFSET,
            entry.extension_block_index as u32,
        )?;

        if entry.extension_block_index == 0 {
            self.release_extension_block(entry)?;
        } else {
            ext_block.write_checksum()?;
        }

        Ok(())
    }

    fn release_extension_block(
        &mut self,
        entry: FileDataBlockListEntry,
    ) -> Result<(), Error> {
        if let Some(last_entry) = self.block_data_list.last() {
            self.fs.borrow_mut().free_block(entry.extension_block_address)?;
            self.update_extension_block_next(
                last_entry.extension_block_address,
                0,
            )?;
        }
        Ok(())
    }

    fn release_data_block(
        &mut self,
        entry: FileDataBlockListEntry,
    ) -> Result<(), Error> {
        self.fs.borrow_mut().free_block(entry.data_block_address)?;
        self.unset_extension_block_data_block(entry)?;

        let block_data_offset = self.size%self.block_data_size;

        if block_data_offset == 0 {
            self.size -= self.block_data_size;
        } else {
            self.size -= block_data_offset;
        }
        self.pos = self.pos.min(self.size);

        Ok(())
    }

    fn init_extension_block(
        &self,
        ext_block_addr: LBAAddress,
    ) -> Result<(), Error> {
        let mut ext_block = Block::new(
            self.fs.borrow().disk(),
            ext_block_addr,
        );

        ext_block.clear()?;
        ext_block.write_block_primary_type(BlockPrimaryType::List)?;
        ext_block.write_block_secondary_type(BlockSecondaryType::File)?;
        ext_block.write_u32(
            BLOCK_DATA_LIST_HEADER_KEY_OFFSET,
            ext_block_addr as u32,
        )?;
        ext_block.write_u32(
            BLOCK_PARENT_OFFSET,
            self.header_block_address as u32,
        )?;
        ext_block.write_checksum()?;

        Ok(())
    }

    fn init_data_block(
        &self,
        block_addr: LBAAddress,
    ) -> Result<(), Error> {
        let mut block = Block::new(
            self.fs.borrow().disk(),
            block_addr,
        );

        block.clear()?;

        if let FilesystemType::OFS = self.fs.borrow().get_filesystem_type()? {
            block.write_block_primary_type(BlockPrimaryType::Data)?;
            block.write_u32(
                BLOCK_DATA_OFS_HEADER_KEY_OFFSET,
                self.header_block_address as u32,
            )?;
            block.write_u32(
                BLOCK_DATA_OFS_SEQ_NUM_OFFSET,
                self.block_data_list.len() as u32 + 1,
            )?;
            block.write_checksum()?;
        }

        Ok(())
    }

    fn alloc_extension_block(
        &mut self,
    ) -> Result<(LBAAddress, usize), Error> {
        match self.block_data_list.last().copied() {
            None => Ok((self.header_block_address, 0)),
            Some(entry) => {
                if entry.extension_block_index < BLOCK_DATA_LIST_SIZE - 1 {
                    Ok((
                        entry.extension_block_address,
                        entry.extension_block_index + 1,
                    ))
                } else {
                    let ext_block_addr = self.fs.borrow_mut().reserve_block()?;

                    self.init_extension_block(ext_block_addr)?;
                    self.update_extension_block_next(
                        entry.extension_block_address,
                        ext_block_addr
                    )?;

                    Ok((ext_block_addr, 0))
                }
            }
        }
    }

    fn alloc_data_block(
        &mut self,
    ) -> Result<FileDataBlockListEntry, Error> {
        let (
            extension_block_address,
            extension_block_index,
        ) = self.alloc_extension_block()?;

        let data_block_address = self.fs.borrow_mut().reserve_block()?;

        self.init_data_block(data_block_address)?;

        let entry = FileDataBlockListEntry {
            data_block_address,
            extension_block_address,
            extension_block_index,
        };

        self.set_extension_block_data_block(entry)?;

        Ok(entry)
    }

    pub(super) fn pop_data_block_list_entry(
        &mut self,
    ) -> Result<(), Error> {
        if let Some(entry) = self.block_data_list.pop() {
            self.release_data_block(entry)?;

            if let FilesystemType::OFS = self.fs.borrow().get_filesystem_type()? {
                if let Some(prev_entry) = self.block_data_list.last() {
                    let mut block = Block::new(
                        self.fs.borrow().disk(),
                        prev_entry.data_block_address,
                    );

                    block.write_u32(BLOCK_DATA_OFS_NEXT_DATA_OFFSET, 0)?;
                    block.write_checksum()?;
                }
            }
        }
        Ok(())
    }

    pub(super) fn push_data_block_list_entry(
        &mut self,
    ) -> Result<FileDataBlockListEntry, Error> {
        let entry = self.alloc_data_block()?;

        if let Some(prev_entry) = self.block_data_list.last() {
            if let FilesystemType::OFS = self.fs.borrow().get_filesystem_type()? {
                let mut block = Block::new(
                    self.fs.borrow().disk(),
                    prev_entry.data_block_address,
                );

                block.write_u32(
                    BLOCK_DATA_OFS_NEXT_DATA_OFFSET,
                    entry.data_block_address as u32,
                )?;
                block.write_checksum()?;
            }
        } else {
            let mut block = Block::new(
                self.fs.borrow().disk(),
                self.header_block_address,
            );

            block.write_u32(
                BLOCK_FIRST_DATA_OFFSET,
                entry.data_block_address as u32,
            )?;
            block.write_checksum()?;
        }

        self.block_data_list.push(entry);
        Ok(entry)
    }

    pub(super) fn get_data_block_list_entry(
        &self,
        pos: usize,
    ) -> Option<FileDataBlockListEntry> {
        self.block_data_list.get(pos/self.block_data_size).copied()
    }

    pub(super) fn sync_all(
        &mut self,
    ) -> Result<(), Error> {
        if test_file_mode(FileMode::Write, self.mode) {
            let mut block = Block::new(
                self.fs.borrow().disk(),
                self.header_block_address,
            );

            block.write_file_size(self.size)?;
            block.write_alteration_date(&SystemTime::now())?;
            block.write_checksum()?;
        }
        Ok(())
    }
}

fn init_file_block_header(
    fs: &AmigaDos,
    name: &str,
) -> Result<LBAAddress, Error> {
    let block_addr = fs.inner.borrow_mut().reserve_block()?;
    let mut block = Block::new(fs.disk(), block_addr);

    block.clear()?;

    block.write_block_primary_type(BlockPrimaryType::Header)?;
    block.write_block_secondary_type(BlockSecondaryType::File)?;
    block.write_alteration_date(&SystemTime::now())?;
    block.write_name(name)?;
    block.write_u32(
        BLOCK_DATA_LIST_HEADER_KEY_OFFSET,
        block.address as u32,
    )?;

    Ok(block_addr)
}

impl File {
    pub(super) fn try_open(
        fs: &AmigaDos,
        path: &Path,
        mode: usize,
    ) -> Result<Self, Error> {
        let metadata =  fs.metadata(path)?;

        if !metadata.is_file() {
            return Err(Error::NotAFileError);
        }

        let header_block_address = metadata.header_block_address();
        let size = metadata.size();
        let pos = 0;

        let block_data_list = FileDataBlockListEntry::try_get_block_data_list(
            fs.disk(),
            header_block_address,
        )?;
        let (
            block_data_offset,
            block_data_size,
        ) = get_data_block_info(fs)?;

        let file = Self {
            fs: fs.inner.clone(),
            block_data_list,
            block_data_offset,
            block_data_size,
            header_block_address,
            mode,
            pos,
            size,
        };

        Ok(file)
    }

    pub(super) fn try_create(
        fs: &AmigaDos,
        path: &Path,
        mode: usize,
        create_new: bool,
    ) -> Result<File, Error> {
        if fs.exists(path)? {
            if create_new {
                return Err(Error::AlreadyExists);
            } else {
                let mut file = File::try_open(fs, path, mode)?;

                file.set_len(0)?;
                return Ok(file);
            }
        }

        let name = get_basename(path)?;
        let parent_path = get_dirname(path)?;

        let block_data_list = Vec::new();
        let (
            block_data_offset,
            block_data_size,
        ) = get_data_block_info(fs)?;

        let mut parent_dir = Dir::try_with_path(fs, parent_path)?;

        let header_block_addr = init_file_block_header(fs, name)?;

        parent_dir.add_entry(name, header_block_addr)?;

        Ok(Self {
            fs: fs.inner.clone(),
            block_data_list,
            block_data_offset,
            block_data_size,
            header_block_address: header_block_addr,
            mode,
            size: 0,
            pos: 0,
        })
    }
}

impl File {
    pub fn options() -> OpenOptions {
        OpenOptions::default()
    }
}
