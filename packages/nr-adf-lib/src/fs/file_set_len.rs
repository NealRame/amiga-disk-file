use crate::block::*;
use crate::errors::*;

use super::amiga_dos_options::*;
use super::constants::*;
use super::file::*;
// use super::file_set_time::*;


impl File {
    fn sync_block(
        &mut self,
        block: &mut Block,
        block_size: usize,
    ) -> Result<(), Error> {
        if let FilesystemType::OFS = self.fs.borrow().get_filesystem_type()? {
            block.write_u32(
                BLOCK_DATA_OFS_SIZE_OFFSET,
                block_size as u32,
            )?;
            block.write_checksum()?;
        }
        Ok(())
    }

    fn grow_block(
        &mut self,
        entry: &FileDataBlockListEntry,
        new_size: usize,
    ) -> Result<(), Error> {
        let mut block = Block::new(
            self.fs.borrow().disk(),
            entry.data_block_address,
        );

        let block_offset = self.size%self.block_data_size;
        let block_size = usize::min(
            new_size - self.size/self.block_data_size,
            self.block_data_size,
        );

        block.fill(0, block_offset, block_offset + block_size)?;

        self.sync_block(&mut block, block_size)?;

        Ok(())
    }

    pub fn grow(
        &mut self,
        new_size: usize,
    ) -> Result<(), Error> {
        assert!(new_size > self.size, "internal error");

        if let Some(entry) = self.get_data_block_list_entry(self.size) {
            self.grow_block(&entry, new_size)?;
            self.size = usize::min(
                (self.size/self.block_data_size + 1)*self.block_data_size,
                new_size,
            );
        }

        while self.size < new_size {
            let entry = self.push_data_block_list_entry()?;

            self.grow_block(&entry, new_size)?;
            self.size = usize::min(
                self.size + self.block_data_size,
                new_size,
            );
        }

        self.sync_all()?;

        Ok(())
    }

    pub(super) fn get_block_index(
        &self,
        size: usize,
    ) -> usize {
        if size < self.block_data_size {
            0
        } else if size%self.block_data_size > 0 {
            size/self.block_data_size
        } else {
            size/self.block_data_size -  1
        }
    }

    fn shrink(
        &mut self,
        new_size: usize,
    ) -> Result<(), Error> {
        assert!(new_size < self.size, "internal error");

        let new_size_block_index = self.get_block_index(new_size);

        while self.get_block_index(self.size) > new_size_block_index {
            self.pop_data_block_list_entry()?;
        }

        if let Some(entry) = self.get_data_block_list_entry(new_size) {
            let block_size = new_size%self.block_data_size;
            if block_size > 0 {
                let mut block = Block::new(
                    self.fs.borrow().disk(),
                    entry.data_block_address,
                );
                self.sync_block(&mut block, block_size)?;
            } else {
                self.pop_data_block_list_entry()?;
            }
        }

        self.size = new_size;
        self.pos = usize::min(
            self.pos,
            self.size,
        );
        self.sync_all()?;

        Ok(())
    }

    pub fn set_len(
        &mut self,
        size: usize,
    ) -> Result<(), Error> {
        check_file_mode(FileMode::Write, self.mode)?;

        if size > self.size {
            return self.grow(size);
        }

        if size < self.size {
            return self.shrink(size);
        }

        self.sync_all()
    }
}
