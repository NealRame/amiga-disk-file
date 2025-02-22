pub mod block;
pub mod disk;
pub mod errors;

pub mod fs;

pub mod prelude;


#[cfg(test)]
mod tests {
    use disk::{
        Disk, DiskType, BLOCK_SIZE, DD_BLOCK_COUNT, HD_BLOCK_COUNT
    };

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
