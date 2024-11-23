use crate::disk::Disk;
use crate::errors::Error;
use crate::fs::boot_block::BootBlock;


pub struct AmigaDos<'disk> {
    pub disk: &'disk Disk,
    pub boot_block: BootBlock,
}

impl<'disk> TryFrom<&'disk Disk> for AmigaDos<'disk> {
    type Error = Error;

    fn try_from(disk: &'disk Disk) -> Result<Self, Self::Error> {
        let boot_block = BootBlock::try_read_from_disk(disk)?;

        Ok(AmigaDos {
            disk,
            boot_block,
        })
    }
}

// impl<'disk> AmigaDos<'disk> {
//     pub fn init(disk: &'disk mut Disk) -> Result<Self, Error> {
//         let bk0 = disk.block_mut(0).or(Err(Error::DiskError))?;

//         bk0.fill(0);

//         // OFS & NO_INTL & NO_DIRC
//         (&mut bk0[0.. 4]).write_all(&[0x44, 0x4f, 0x53, 0]).or(Err(Error::DiskError))?;
//         (&mut bk0[8..12]).write_all(& 880u32.to_be_bytes()).or(Err(Error::DiskError))?;

//         disk.block_mut(1)
//             .and_then(|bk| Ok(bk.fill(0)))
//             .or(Err(Error::DiskError))?;

//         return Ok(Self {
//             disk
//         })
//     }
// }
