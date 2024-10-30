use std::fmt;


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
