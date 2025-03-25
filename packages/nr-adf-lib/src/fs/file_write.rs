use std::path::Path;
use std::time::SystemTime;

use crate::block::*;
use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::amiga_dos_options::*;
use super::constants::*;
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
        let mut block = Block::new(
            self.fs.borrow().disk(),
            data_list_entry.data_block_address,
        );

        block.write_u8_array(self.block_data_offset + data_pos, buf)?;

        if let FilesystemType::OFS = self.fs.borrow().get_filesystem_type()? {
            let old_data_size = block.read_u32(BLOCK_DATA_OFS_SIZE_OFFSET)?;
            let new_data_size = u32::max(
                old_data_size,
                (data_pos + buf.len()) as u32,
            );

            block.write_u32(BLOCK_DATA_OFS_SIZE_OFFSET, new_data_size)?;
            block.write_checksum()?;
        }

        Ok(())
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

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::cell::RefCell;

    use crate::fs::*;
    use super::*;

    fn write_ofs(count: usize) {
        let data_in = vec![42u8; count];

        let disk = Disk::create(DiskType::DoubleDensity);
        let fs = AmigaDosFormater::default()
            .with_cache_mode(CacheMode::Off)
            .with_filesystem_type(FilesystemType::OFS)
            .with_international_mode(InternationalMode::Off)
            .format(Rc::new(RefCell::new(disk)), "TEST")
            .unwrap();

        fs.write("/data", &data_in).unwrap();

        let data_out = fs.read("/data").unwrap();

        assert_eq!(&data_in, &data_out);
    }

    #[test]
    fn write_ofs_less_than_488_bytes() {
        write_ofs(488)
    }

    #[test]
    fn write_ofs_more_than_488_bytes() {
        write_ofs(489)
    }

    #[test]
    fn write_ofs_less_than_15128_bytes() {
        write_ofs(15128)
    }

    #[test]
    fn write_ofs_more_than_15129_bytes() {
        write_ofs(15129)
    }

    #[test]
    fn write_ofs_less_than_35136_bytes() {
        write_ofs(35136)
    }

    #[test]
    fn write_ofs_more_than_35136_bytes() {
        write_ofs(35137)
    }
}
