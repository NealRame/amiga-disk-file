use std::path::Path;
use std::time::SystemTime;

use crate::block::*;
use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::file::*;


impl File {
    fn sync_all(
        &mut self,
    ) -> Result<(), Error> {
        if test_file_mode(FileMode::Write, self.mode) {
            let mut block = Block::new(
                self.fs.borrow().disk(),
                self.header_block_addr,
            );

            block.write_file_size(self.size)?;
            block.write_alteration_date(&SystemTime::now())?;
            block.write_checksum()?;
        }
        Ok(())
    }

    fn write_data(
        &mut self,
        buf: &[u8],
        data_list_entry: &FileDataBlockListEntry,
        data_pos: usize,
    ) -> Result<(), Error> {
        Block::new(
            self.fs.borrow().disk(),
            data_list_entry.data_block_address,
        ).write_u8_array(self.block_data_offset + data_pos, buf)
    }

    pub fn write(
        &mut self,
        mut buf: &[u8],
    ) -> Result<usize, Error> {
        check_file_mode(FileMode::Write, self.mode)?;

        let mut count = 0;

        while !buf.is_empty() {
            let data_pos = self.pos%self.block_data_size;
            let data_len
                = buf.len()
                    // .min(self.size - self.pos)
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
            self.size = self.pos.max(self.size);
        }

        self.sync_all()?;

        Ok(count)
    }
}

impl AmigaDos {
    /// Writes a slice as the entire contents of a file.
    /// This function will create a file if it does not exist, and will
    /// entirely replace its contents if it does.
    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        path: P,
        data: C
    ) -> Result<(), Error> {
        let mut output = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(
                self,
                path.as_ref(),
            )?;

        for chunk in data.as_ref().chunks(BLOCK_SIZE) {
            output.write(chunk)?;
        }

        Ok(())
    }
}
