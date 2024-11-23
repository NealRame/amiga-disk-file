use std::fmt;


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    CorruptedImageFile,
    DiskError,
}

impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
