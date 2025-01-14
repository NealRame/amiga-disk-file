use std::path::Path;

use crate::errors::*;

use super::amiga_dos::*;
use super::block::BlockWriter;
use super::file::*;


impl File {
    fn overwrite_data(
        &mut self,
        buf: &[u8],
    ) -> Result<usize, Error> {
        let block_addr = self.get_data_block_addr()?;

        let data_pos = self.pos%self.block_data_size;
        let data_len = buf.len()
            .min(self.size - self.pos)
            .min(self.block_data_size - data_pos);

        let mut fs = self.fs.borrow_mut();
        let disk = fs.disk_mut();

        BlockWriter::try_from_disk(
            disk,
            block_addr,
        )?.write_u8_array(
            self.block_data_offset + data_pos,
            &buf[..data_len],
        )?;
        Ok(data_len)
    }

    fn write_data(
        &mut self,
        buf: &[u8],
    ) -> Result<usize, Error> {
        let block_addr = self.get_new_data_block_addr()?;

        let data_pos = self.pos%self.block_data_size;
        let data_len = buf.len().min(self.block_data_size - data_pos);

        let mut fs = self.fs.borrow_mut();
        let disk = fs.disk_mut();

        BlockWriter::try_from_disk(
            disk,
            block_addr,
        )?.write_u8_array(
            self.block_data_offset + data_pos,
            &buf[..data_len],
        )?;
        Ok(data_len)
    }

    pub fn write(
        &mut self,
        mut buf: &[u8],
    ) -> Result<usize, Error> {
        if ! self.mode & FileMode::Write {
            return Err(Error::BadFileDescriptor);
        }

        let mut total = 0;

        while buf.len() > 0 {
            let n = if self.pos < self.size {
                // overwrite data
                self.overwrite_data(buf)?
            } else {
                // write new data
                self.write_data(buf)?
            };

            buf = &buf[n..];
            total += n;

            self.pos += n;
            self.size = self.size.max(self.pos);
        }

        Ok(total)
    }
}

impl AmigaDos {
    /// Writes a slice as the entire contents of a file.
    /// This function will create a file if it does not exist, and will
    /// entirely replace its contents if it does.
    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        path: P,
        contents: C
    ) -> Result<(), Error> {
        unimplemented!()
    }
}
