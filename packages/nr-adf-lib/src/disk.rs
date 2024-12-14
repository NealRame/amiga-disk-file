use crate::errors::*;

pub const BLOCK_SIZE     : usize =  512;
pub const DD_BLOCK_COUNT : usize = 1760;
pub const HD_BLOCK_COUNT : usize = 3520;

pub type LBAAddress = usize;

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiskType {
    DoubleDensity = DD_BLOCK_COUNT,
    HighDensity   = HD_BLOCK_COUNT,
}

impl DiskType {
    pub fn size(self) -> usize {
        (self as usize)*BLOCK_SIZE
    }
}

pub struct Disk {
    disk_data: Vec<u8>,
    disk_type: DiskType,
}

impl Disk {
    fn block_bounds(
        &self,
        addr: LBAAddress,
        count: usize,
    ) -> Result<(usize, usize), Error> {
        if addr >= self.block_count() || addr + count > self.block_count() {
            Err(Error::DiskInvalidLBAAddressError(addr))
        } else {
            Ok((addr*BLOCK_SIZE, (addr + count)*BLOCK_SIZE))
        }
    }
}

impl Disk {
    pub fn block_count(&self) -> usize {
        self.disk_type as usize
    }

    pub fn size(&self) -> usize {
        self.block_count()*BLOCK_SIZE
    }

    pub fn disk_type(&self) -> DiskType {
        self.disk_type
    }
}

impl Disk {
    pub fn create(disk_type: DiskType) -> Self {
        let disk_data = vec![0; disk_type.size()];

        Self {
            disk_data,
            disk_type,
        }
    }

    pub fn try_create_with_data(
        disk_data: Vec<u8>,
    ) -> Result<Self, Error> {
        let disk_size = disk_data.len();

        if disk_size == DiskType::DoubleDensity.size() {
            return Ok(Disk {
                disk_data,
                disk_type: DiskType::DoubleDensity,
            });
        }

        if disk_size == DiskType::HighDensity.size() {
            return Ok(Disk {
                disk_data,
                disk_type: DiskType::HighDensity,
            });
        }

        Err(Error::DiskInvalidSizeError(disk_size))
    }

    pub fn block(
        &self,
        addr: LBAAddress,
    ) -> Result<&[u8], Error> {
        let (begin, end) = self.block_bounds(addr, 1)?;

        Ok(&self.disk_data[begin..end])
    }

    pub fn block_mut(
        &mut self,
        addr: LBAAddress,
    ) -> Result<&mut [u8], Error> {
        let (begin, end) = self.block_bounds(addr, 1)?;

        Ok(&mut self.disk_data[begin..end])
    }

    pub fn data(&self) -> &[u8] {
        self.disk_data.as_slice()
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        self.disk_data.as_mut_slice()
    }

    pub fn read_blocks(
        &self,
        addr: LBAAddress,
        count: usize,
    ) -> Result<Vec<u8>, Error> {
        let first = addr;
        let last = addr + count;
        let mut data = Vec::with_capacity(addr*BLOCK_SIZE);

        for addr in first..last {
            data.extend_from_slice(self.block(addr)?);
        }
        Ok(data)
    }
}
