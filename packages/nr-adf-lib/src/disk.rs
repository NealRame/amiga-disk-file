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

#[derive(Debug)]
pub struct Disk {
    disk_data: Vec<u8>,
    disk_type: DiskType,
}

impl Disk {
    fn block_bounds(
        &self,
        addr: LBAAddress,
        count: usize,
    ) -> Result<std::ops::Range<usize>, Error> {
        if addr >= self.block_count() || addr + count > self.block_count() {
            Err(Error::DiskInvalidLBAAddressError(addr))
        } else {
            let begin = addr*BLOCK_SIZE;
            let end = (addr + count)*BLOCK_SIZE;
            Ok(begin..end)
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

    pub fn blocks(
        &self,
        addr: LBAAddress,
        count: usize,
    ) -> Result<&[u8], Error> {
        let r = self.block_bounds(addr, count)?;
        Ok(&self.disk_data[r])
    }

    pub fn blocks_mut(
        &mut self,
        addr: LBAAddress,
        count: usize,
    ) -> Result<&mut [u8], Error> {
        let r = self.block_bounds(addr, count)?;
        Ok(&mut self.disk_data[r])
    }

    pub fn data(&self) -> &[u8] {
        self.disk_data.as_slice()
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        self.disk_data.as_mut_slice()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dd_floppy_is_ok() {
        let disk = Disk::create(DiskType::DoubleDensity);

        assert_eq!(disk.block_count(), DD_BLOCK_COUNT);
        assert_eq!(disk.size(), DD_BLOCK_COUNT*BLOCK_SIZE);
    }

    #[test]
    fn hd_floppy_is_ok() {
        let disk = Disk::create(DiskType::HighDensity);

        assert_eq!(disk.block_count(), HD_BLOCK_COUNT);
        assert_eq!(disk.size(), HD_BLOCK_COUNT*BLOCK_SIZE);
    }
}
