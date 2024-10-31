use std::fmt;

use crate::disk::DiskGeometry;

/******************************************************************************
 * InvalidCHSAddressError
 *****************************************************************************/
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidCHSAddressError(usize, usize, usize);

impl From<(usize, usize, usize)> for InvalidCHSAddressError {
    fn from(value: (usize, usize, usize)) -> Self {
        InvalidCHSAddressError(value.0, value.1, value.2)
    }
}

impl std::error::Error for InvalidCHSAddressError {}
impl fmt::Display for InvalidCHSAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/******************************************************************************
 * InvalidLBAAddressError
 *****************************************************************************/
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidLBAAddressError(usize);

impl From<usize> for InvalidLBAAddressError {
    fn from(value: usize) -> Self {
        InvalidLBAAddressError(value)
    }
}

impl std::error::Error for InvalidLBAAddressError {}
impl fmt::Display for InvalidLBAAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/******************************************************************************
 * InvalidSizeError
 *****************************************************************************/
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidSizeError(DiskGeometry);

impl From<DiskGeometry> for InvalidSizeError {
    fn from(value: DiskGeometry) -> Self {
        InvalidSizeError(value)
    }
}

impl std::error::Error for InvalidSizeError {}
impl fmt::Display for InvalidSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
