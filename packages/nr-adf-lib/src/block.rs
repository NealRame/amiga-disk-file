use std::cell::RefCell;
use std::rc::Rc;

use paste::paste;

use crate::disk::{
    BLOCK_SIZE,
    Disk,
    LBAAddress,
};

use crate::errors::Error;


pub struct Block {
    pub disk: Rc<RefCell<Disk>>,
    pub address: LBAAddress,
}

impl Block {
    pub fn new(
        disk: Rc<RefCell<Disk>>,
        address: LBAAddress,
    ) -> Self {
        Self {
            disk,
            address,
        }
    }
}

impl Block {
    pub fn read_u8(
        &self,
        offset: usize,
    ) -> Result<u8, Error> {
        let disk = self.disk.borrow();
        let disk_data = disk.blocks(self.address, 1)?;

        if offset < disk_data.len() {
            Ok(disk_data[offset])
        } else {
            Err(Error::DiskInvalidBlockOffsetError(offset))
        }
    }

    pub fn read_u8_array(
        &self,
        offset: usize,
        data: &mut [u8],
    ) -> Result<(), Error> {
        let disk = self.disk.borrow();
        let disk_data = disk.blocks(self.address, 1)?;

        if offset + data.len() <= disk_data.len() {
            data.copy_from_slice(&disk_data[offset..offset + data.len()]);
            Ok(())
        } else {
            Err(Error::DiskInvalidBlockOffsetError(offset))
        }
    }

    pub fn read_u8_vector(
        &self,
        offset: usize,
        len: usize,
    ) -> Result<Vec<u8>, Error> {
        let mut v = vec![0; len];

        self.read_u8_array(offset, &mut v)?;
        Ok(v)
    }
}

impl Block {
    pub fn write_u8(
        &mut self,
        offset: usize,
        value: u8,
    ) -> Result<(), Error> {
        let mut disk = self.disk.borrow_mut();
        let disk_data = disk.blocks_mut(self.address, 1)?;

        if offset < disk_data.len() {
            disk_data[offset] = value;
            Ok(())
        } else {
            Err(Error::DiskInvalidBlockOffsetError(offset))
        }
    }

    pub fn write_u8_array(
        &mut self,
        offset: usize,
        values: &[u8],
    ) -> Result<(), Error> {
        let mut disk = self.disk.borrow_mut();
        let disk_data = disk.blocks_mut(self.address, 1)?;
        let size = values.len();

        if offset + size <= disk_data.len() {
            disk_data[offset..offset + size].copy_from_slice(values);
            Ok(())
        } else {
            Err(Error::DiskInvalidBlockOffsetError(offset))
        }
    }
}

macro_rules! generate_block_read_write_fns {
    ($($t:ty),*) => {

        paste! {$(impl Block {
            pub fn [<read_ $t>](
                &self,
                offset: usize,
            ) -> Result<$t, Error> {
                let disk = self.disk.borrow();
                let disk_data = disk.blocks(self.address, 1)?;
                let size = std::mem::size_of::<$t>();

                if let Ok(buf) = disk_data[offset..offset + size].try_into() {
                    Ok($t::from_be_bytes(buf))
                } else {
                    Err(Error::DiskInvalidBlockOffsetError(offset))
                }
            }

            pub fn [<read_ $t _array>](
                &self,
                offset: usize,
                values: &mut [$t],
            ) -> Result<(), Error> {
                let size = std::mem::size_of::<$t>();

                for i in 0..values.len() {
                    values[i] = self.[<read_ $t>](offset + i*size)?
                }
                Ok(())
            }

            pub fn [<read_ $t _vector>](
                &self,
                offset: usize,
                len: usize,
            ) -> Result<Vec<$t>, Error> {
                let mut v = Vec::new();

                v.resize(len, 0);
                self.[<read_ $t _array>](offset, &mut v)?;
                Ok(v)
            }

            pub fn [<write_ $t>](
                &mut self,
                offset: usize,
                value: $t,
            ) -> Result<(), Error> {
                let mut disk = self.disk.borrow_mut();
                let disk_data = disk.blocks_mut(self.address, 1)?;

                let size = std::mem::size_of::<$t>();
                let end = offset + size;

                if end <= disk_data.len() {
                    let slice = &mut disk_data[offset..end];
                    slice.copy_from_slice(&value.to_be_bytes());
                    Ok(())
                } else {
                    Err(Error::DiskInvalidBlockOffsetError(offset))
                }
            }

            // pub fn [<write_ $t _array>](
            //     &mut self,
            //     offset: usize,
            //     values: &[$t],
            // ) -> Result<(), Error> {
            //     for i in 0..values.len() {
            //         self.[<write_ $t>](offset + i, values[i])?
            //     }
            //     Ok(())
            // }
        })*}
    };
}

generate_block_read_write_fns!(u32);

impl Block {
    pub fn read_string(
        &self,
        offset: usize,
        len: usize,
    ) -> Result<String, Error> {
        let bytes = self.read_u8_vector(offset, len)?;

        if let Ok(s) = String::from_utf8(bytes) {
            Ok(s)
        } else {
            Err(Error::InvalidStringError)
        }
    }
}

impl Block {
    pub fn write_string(
        &mut self,
        offset: usize,
        s: &str,
    ) -> Result<(), Error> {
        self.write_u8_array(offset, s.as_bytes())
    }
}

impl Block {
    pub fn clear(
        &mut self,
    ) -> Result<(), Error> {
        self.fill(0, 0, BLOCK_SIZE)
    }

    pub fn fill(
        &mut self,
        value: u8,
        from: usize,
        to: usize,
    ) -> Result<(), Error> {
        let mut disk = self.disk.borrow_mut();
        let disk_data = disk.blocks_mut(self.address, 1)?;

        disk_data[from..to].fill(value);
        Ok(())
    }
}
