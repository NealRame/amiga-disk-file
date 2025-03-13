use nr_adf_lib::disk::DiskType;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum ArgDiskType {
    DD,
    HD,
}

impl From<ArgDiskType> for DiskType {
    fn from(value: ArgDiskType) -> Self {
        match value {
            ArgDiskType::DD => Self::DoubleDensity,
            ArgDiskType::HD => Self::HighDensity,
        }
    }
}
