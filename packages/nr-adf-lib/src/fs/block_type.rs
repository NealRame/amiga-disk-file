use crate::errors::Error;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockPrimaryType {
    Header = 2,
    Data   = 8,
    List   = 16,
}

impl Into<u32> for BlockPrimaryType {
    fn into(self) -> u32 {
        self as u32
    }
}

impl TryFrom<u32> for BlockPrimaryType {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            _ if value == BlockPrimaryType::Header.into() => {
                Ok(BlockPrimaryType::Header)
            },
            _ if value == BlockPrimaryType::Data.into() => {
                Ok(BlockPrimaryType::Data)
            },
            _ if value == BlockPrimaryType::List.into() => {
                Ok(BlockPrimaryType::List)
            },
            _  => Err(Error::InvalidFilesystemBlockPrimaryTypeError(value)),
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockSecondaryType {
    Root              = 1,
    Directory         = 2,
    SoftLink          = 3,
    HardLinkDirectory = 4,
    HardLinkFile      = u32::MAX - 4 + 1,
    File              = u32::MAX - 3 + 1,
}

impl Into<u32> for BlockSecondaryType {
    fn into(self) -> u32 {
        self as u32
    }
}

impl TryFrom<u32> for BlockSecondaryType {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            _ if value == BlockSecondaryType::Root.into() => {
                Ok(BlockSecondaryType::Root)
            },
            _ if value == BlockSecondaryType::Directory.into() => {
                Ok(BlockSecondaryType::Directory)
            },
            _ if value == BlockSecondaryType::SoftLink.into() => {
                Ok(BlockSecondaryType::SoftLink)
            },
            _ if value == BlockSecondaryType::HardLinkDirectory.into() => {
                Ok(BlockSecondaryType::HardLinkDirectory)
            },
            _ if value == BlockSecondaryType::HardLinkFile.into() => {
                Ok(BlockSecondaryType::HardLinkFile)
            },
            _ if value == BlockSecondaryType::File.into() => {
                Ok(BlockSecondaryType::File)
            },
            _  => Err(Error::InvalidFilesystemBlockSecondaryTypeError(value)),
        }
    }
}
