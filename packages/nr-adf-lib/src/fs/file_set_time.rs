use std::time::SystemTime;

use crate::block::*;
use crate::errors::*;

use super::file::*;


impl File {
    pub fn set_time(
        &mut self,
        datetime: &SystemTime,
    ) -> Result<(), Error> {
        check_file_mode(FileMode::Write, self.mode)?;

        let mut block = Block::new(
            self.fs.borrow().disk(),
            self.header_block_address,
        );

        block.write_alteration_date(datetime)?;
        block.write_checksum()?;

        Ok(())
    }
}
