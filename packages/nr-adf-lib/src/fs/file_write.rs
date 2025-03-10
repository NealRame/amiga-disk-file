use crate::block::*;
use crate::errors::*;

use super::file::*;


impl File {
    fn write_data(
        &mut self,
        buf: &[u8],
        data_list_entry: &FileDataBlockListEntry,
        data_pos: usize,
    ) -> Result<(), Error> {
        Block::new(
            self.fs.borrow().disk(),
            data_list_entry.data_block_address,
        ).write_u8_array(data_pos, buf)
    }

    pub fn write(
        &mut self,
        mut buf: &[u8],
    ) -> Result<usize, Error> {
        check_file_mode(FileMode::Write, self.mode)?;

        let mut count = 0;

        while buf.len() > 0 {
            let data_pos = self.pos%self.block_data_size;
            let data_len
                = buf.len()
                    .min(self.size - self.pos)
                    .min(self.block_data_size - data_pos);

            let data_list_entry =
                if let Some(entry) = self.get_data_block_list_entry(self.pos) {
                    entry
                } else {
                    self.push_data_block_list_entry()?
                };

            self.write_data(&buf[..data_len], &data_list_entry, data_pos)?;

            buf = &buf[data_len..];
            count += data_len;

            self.pos += data_len;
            self.size = self.pos.min(self.size);
        }

        Ok(count)
    }
}
