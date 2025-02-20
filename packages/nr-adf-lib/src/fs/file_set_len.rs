use crate::errors::*;

use super::block::*;
use super::file::*;


impl File {
    pub fn grow(
        &mut self,
        new_size: usize,
    ) -> Result<(), Error> {
        assert!(new_size > self.size, "internal error");

        if let Some(entry) = self.get_data_block_list_entry(self.size) {
            Block::new(
                self.fs.borrow().disk(),
                entry.data_block_address,
            ).fill(0, self.size%self.block_data_size, self.block_data_size)?;

            self.size = usize::min(
                (self.size/self.block_data_size + 1)*self.block_data_size,
                new_size,
            );
        }

        while self.size < new_size {
            let entry = self.push_data_block_list_entry()?;

            Block::new(
                self.fs.borrow().disk(),
                entry.data_block_address,
            ).clear()?;

            self.size = usize::min(
                self.size + self.block_data_size,
                new_size,
            );
        }

        Ok(())
    }

    fn shrink(
        &mut self,
        new_size: usize,
    ) -> Result<(), Error> {
        assert!(new_size < self.size, "internal error");

        let new_size_block_index = new_size/self.block_data_size;

        while self.size/self.block_data_size > new_size_block_index {
            self.pop_data_block_list_entry()?;
        }

        if new_size%self.block_data_size > 0 {
            self.size = new_size;
            self.pos = usize::min(
                self.pos,
                self.size,
            );
        } else {
            self.pop_data_block_list_entry()?;
        }

        Ok(())
    }

    pub fn set_len(
        &mut self,
        size: usize,
    ) -> Result<(), Error> {
        if size > self.size {
            return self.grow(size);
        }
        if size < self.size {
            return self.shrink(size);
        }
        Ok(())
    }
}
