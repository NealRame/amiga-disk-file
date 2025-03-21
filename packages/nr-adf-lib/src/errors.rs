use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    BadFileDescriptor,

    CorruptedImageFile,

    InvalidDataBlockIndexError(usize),
    InvalidHashError(usize),
    InvalidNameError,
    InvalidNameLengthError(usize),
    InvalidPathError,
    InvalidStringError,
    InvalidFileModeError,
    InvalidFilesystemTypeError,
    InvalidCacheModeError,
    InvalidInternationalModeError,

    FileEOF,

    InvalidFilesystemBlockPrimaryTypeError(u32),
    UnexpectedFilesystemBlockPrimaryTypeError(u32),

    InvalidFilesystemBlockSecondaryTypeError(u32),
    UnexpectedFilesystemBlockSecondaryTypeError(u32),

    AlreadyExists,

    NotADirectoryError,
    NotFoundError,

    NoSpaceLeft,

    DiskInvalidLBAAddressError(usize),
    DiskInvalidBlockOffsetError(usize),
    DiskInvalidSizeError(usize),
}

impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
