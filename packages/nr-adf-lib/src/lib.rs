pub mod disk;
pub mod errors;

#[cfg(test)]
mod tests {
    use disk::{DiskGeometry, DiskType};

    use super::*;

    #[test]
    fn dd_floppy_geometry_is_ok() {
        let disk_geometry = DiskGeometry::from(DiskType::DoubleDensity);

        assert_eq!(disk_geometry.block_count, 1760);
        assert_eq!(disk_geometry.block_size, 512);
        assert_eq!(disk_geometry.size(), 901_120);
    }

    #[test]
    fn hd_floppy_geometry_is_ok() {
        let disk_geometry = DiskGeometry::from(DiskType::HighDensity);

        assert_eq!(disk_geometry.block_count, 3520);
        assert_eq!(disk_geometry.block_size, 512);
        assert_eq!(disk_geometry.size(), 1_802_240);
    }
}
