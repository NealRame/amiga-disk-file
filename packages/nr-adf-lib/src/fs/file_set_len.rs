use crate::errors::*;

use super::file::*;


impl File {
    fn shrink(
        &mut self,
        new_size: usize,
    ) -> Result<(), Error> {
        let cur_size = self.size;

        assert!(new_size < self.size, "internal error");

        let cur_size_block_count = cur_size/self.block_data_size;
        let new_size_block_count = new_size/self.block_data_size;



        Ok(())
    }

    fn grow(
        &mut self,
        _: usize,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn set_len(
        &mut self,
        size: usize,
    ) -> Result<(), Error> {
        if size > self.size {
            self.grow(size)
        } else {
            self.shrink(size)
        }
    }
}
