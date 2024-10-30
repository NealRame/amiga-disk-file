use crate::errors::*;


pub type CHSAddress = (usize, usize, usize);
pub type LBAAddress = usize;

pub enum FloppyDiskFormat {
    DoubleDensity,
    HighDensity,
}

pub struct DiskGeometry {
    sector_size: usize,
    sectors_per_track: usize,
    tracks_per_cylinder: usize,
    cylinder_count: usize,
    max_block: usize,
    size: usize,
}

impl From<FloppyDiskFormat> for DiskGeometry {
    fn from(value: FloppyDiskFormat) -> Self {
        match value {
            FloppyDiskFormat::DoubleDensity => DiskGeometry::new(
                512, // sector size
                11,  // sectors per track
                2,   // tracks per cylinder
                80,  // cylinder count
            ),
            FloppyDiskFormat::HighDensity => DiskGeometry::new(
                512, // sector size
                22,  // sectors per track
                2,   // tracks per cylinder
                80,  // cylinder count
            ),
        }
    }
}

impl DiskGeometry {
    fn new(
        sector_size: usize,
        sectors_per_track: usize,
        tracks_per_cylinder: usize,
        cylinder_count: usize,
    ) -> Self {
        let max_block = sectors_per_track*tracks_per_cylinder*cylinder_count;
        let size = max_block*sector_size;

        Self {
            sector_size,
            sectors_per_track,
            tracks_per_cylinder,
            cylinder_count,
            max_block,
            size,
        }
    }
}

impl DiskGeometry {
    pub fn sector_size(&self) -> usize {
        self.sector_size
    }

    pub fn sector_per_track(&self) -> usize {
        self.sectors_per_track
    }

    pub fn max_block(&self) -> usize {
        self.max_block
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn try_get_lba_address(
        &self,
        addr: CHSAddress,
    ) -> Result<LBAAddress, InvalidCHSAddressError> {
        let (c, h, s) = addr;

        if c < self.cylinder_count
            && h < self.tracks_per_cylinder
            && s < self.sectors_per_track
        {
            Ok(s + self.sectors_per_track*(h + self.tracks_per_cylinder*c))
        } else {
            Err(addr.into())
        }
    }

    pub fn try_get_chs_address(
        &self,
        addr: LBAAddress,
    ) -> Result<CHSAddress, InvalidLBAAddressError> {
        if addr < self.max_block {
            let s = addr%self.sectors_per_track;
            let c = (addr - s)/(self.sectors_per_track*self.tracks_per_cylinder);
            let h = (addr - s - c*self.sectors_per_track*self.tracks_per_cylinder)/self.sectors_per_track;

            Ok((c, h, s))
        } else {
            Err(addr.into())
        }
    }
}
