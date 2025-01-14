use std::path::Path;

use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::block::*;
use super::file::*;


impl File {
    fn read_data(
        &mut self,
        buf: &mut [u8],
        block_data_pos: usize,
    ) -> Result<(), Error> {
        let fs = self.fs.borrow();
        let disk = fs.disk();

        let block_addr = self.get_data_block_addr()?;
        let block = BlockReader::try_from_disk(disk, block_addr)?;

        block.read_u8_array(self.block_data_offset + block_data_pos, buf)
    }

    pub fn read(
        &mut self,
        mut buf: &mut [u8],
    ) -> Result<usize, Error> {
        if ! self.mode & FileMode::Read {
            return Err(Error::BadFileDescriptor);
        }

        if self.pos >= self.size {
            return Ok(0);
        }

        let total = buf.len();
        let mut count = 0;

        while count < total && self.pos < self.size {
            let read_data_pos = self.pos%self.block_data_size;
            let read_data_len =
                buf.len()
                    .min(self.size - self.pos)
                    .min(self.block_data_size - read_data_pos);

            self.read_data(&mut buf[..read_data_len], read_data_pos)?;
            self.pos += read_data_len;

            count += read_data_len;
            buf = &mut buf[read_data_len..];
        }

        Ok(count)
    }
}

impl AmigaDos {
    /// Reads the entire contents of a file into a bytes vector.
    pub fn read<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Vec<u8>, Error> {
        let mut buf = [0; BLOCK_SIZE];

        let mut output = Vec::new();
        let mut input = self.open(&path, 0|FileMode::Read)?;

        loop {
            let count = input.read(&mut buf)?;

            if count > 0 {
                output.extend_from_slice(&buf[..count]);
            } else {
                break
            }
        }

        Ok(output)
    }
}
