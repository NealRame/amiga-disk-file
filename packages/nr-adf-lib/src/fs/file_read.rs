use crate::block::*;
use crate::errors::*;

use super::file::*;


impl File {
    fn read_data(
        &mut self,
        buf: &mut [u8],
        data_list_entry: &FileDataBlockListEntry,
        data_pos: usize,
    ) -> Result<(), Error> {
        let data_block = Block::new(
            self.fs.borrow().disk(),
            data_list_entry.data_block_address,
        );

        data_block.read_u8_array(self.block_data_offset + data_pos, buf)?;
        self.pos += buf.len();

        Ok(())
    }

    pub fn read(
        &mut self,
        mut buf: &mut [u8],
    ) -> Result<usize, Error> {
        check_file_mode(FileMode::Read, self.mode)?;

        if self.pos >= self.size {
            return Ok(0);
        }

        let total = buf.len();
        let mut count = 0;

        while count < total && self.pos < self.size {
            let data_pos = self.pos%self.block_data_size;
            let data_len =
                buf.len()
                    .min(self.size - self.pos)
                    .min(self.block_data_size - data_pos);

            if let Some(entry) = self.get_data_block_list_entry(self.pos) {
                self.read_data(&mut buf[..data_len], &entry, data_pos)?;

                buf = &mut buf[data_len..];
                count += data_len;
            }
        }
        Ok(count)
    }
}
