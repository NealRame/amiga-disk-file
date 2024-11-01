use crate::errors::*;


pub type LBAAddress = usize;

pub enum DiskType {
    DoubleDensity,
    HighDensity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiskGeometry {
    pub block_count: usize,
    pub block_size: usize,
}

impl DiskGeometry {
    pub fn size(&self) -> usize {
        self.block_count*self.block_size
    }
}

impl From<DiskType> for DiskGeometry {
    fn from(value: DiskType) -> Self {
        match value {
            DiskType::DoubleDensity => DiskGeometry {
                block_count: 1760,
                block_size: 512,
            },
            DiskType::HighDensity => DiskGeometry {
                block_count: 3520,
                block_size: 512,
            },
        }
    }
}

pub struct Disk {
    data: Vec<u8>,
    geometry: DiskGeometry,
}

impl Disk {
    fn block_bounds(
        &self,
        addr: LBAAddress,
    ) -> Result<(usize, usize), InvalidLBAAddressError> {
        if addr < self.geometry.block_count {
            let begin = addr*self.geometry.block_size;
            let end = begin + self.geometry.block_size;

            Ok((begin, end))
        } else {
            Err(addr.into())
        }
    }
}

impl Disk {
    pub fn create(disk_type: DiskType) -> Self {
        let disk_geometry = DiskGeometry::from(disk_type);
        let disk_data = vec![0; disk_geometry.size()];

        Self {
            data: disk_data,
            geometry: disk_geometry,
        }
    }

    pub fn try_create_with_data(
        disk_data: Vec<u8>,
        disk_type: DiskType,
    ) -> Result<Self, InvalidSizeError> {
        let disk_geometry = DiskGeometry::from(disk_type);

        if disk_data.len() == disk_geometry.size() {
            Ok(Disk {
                data: disk_data,
                geometry: disk_geometry,
            })
        } else {
            Err(disk_geometry.into())
        }
    }

    pub fn block(
        &self,
        addr: LBAAddress,
    ) -> Result<&[u8], InvalidLBAAddressError> {
        let (begin, end) = self.block_bounds(addr)?;

        Ok(&self.data[begin..end])
    }

    pub fn block_mut(
        &mut self,
        addr: LBAAddress,
    ) -> Result<&mut [u8], InvalidLBAAddressError> {
        let (begin, end) = self.block_bounds(addr)?;

        Ok(&mut self.data[begin..end])
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }
}
