use crate::errors::*;

use super::file::*;


impl File {
    fn shrink(
        &mut self,
        _: usize,
    ) -> Result<(), Error> {
        unimplemented!()
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
