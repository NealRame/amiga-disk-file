use std::fmt;


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
pub struct InvalidSizeError(usize);

impl From<usize> for InvalidSizeError {
    fn from(value: usize) -> Self {
        InvalidSizeError(value)
    }
}

impl std::error::Error for InvalidSizeError {}
impl fmt::Display for InvalidSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
