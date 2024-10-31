pub mod disk;
pub mod errors;

#[cfg(test)]
mod tests {
    use disk::{DiskGeometry, DiskType};

    use super::*;

    #[test]
    fn dd_floppy_disk_max_block_is_ok() {
        let disk_geometry = DiskGeometry::from(DiskType::DoubleDensity);

        assert_eq!(disk_geometry.max_block(), 1760);
    }

    #[test]
    fn dd_floppy_disk_size_is_ok() {
        let disk_geometry = DiskGeometry::from(DiskType::DoubleDensity);

        assert_eq!(disk_geometry.size(), 901120);
    }
}
