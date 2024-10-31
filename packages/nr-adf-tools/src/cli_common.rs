use nr_adf_lib::disk::DiskType;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum ArgDiskType {
    DD,
    HD,
}

impl Into<DiskType> for ArgDiskType {
    fn into(self) -> DiskType {
        match self {
            Self::DD => DiskType::DoubleDensity,
            Self::HD => DiskType::HighDensity,
        }
    }
}
