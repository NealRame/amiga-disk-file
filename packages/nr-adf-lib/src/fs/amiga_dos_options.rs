use std::fmt;
use std::str::FromStr;

use crate::errors::Error;


#[derive(Clone, Copy, Debug, Default)]
#[repr(u8)]
pub enum FilesystemType {
    #[default]
    OFS = 0,
    FFS = 0x01,
}

impl FromStr for FilesystemType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ofs" => Ok(FilesystemType::OFS),
            "ffs" => Ok(FilesystemType::FFS),
            _ => Err(Error::InvalidFilesystemTypeError),
        }
    }
}

impl fmt::Display for FilesystemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilesystemType::OFS => write!(f, "OFS"),
            FilesystemType::FFS => write!(f, "FFS"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(u8)]
pub enum InternationalMode {
    #[default]
    Off = 0,
    On  = 0x02,
}

impl FromStr for InternationalMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "on"|"yes" => Ok(InternationalMode::On),
            "off"|"no" => Ok(InternationalMode::Off),
            _ => Err(Error::InvalidInternationalModeError)
        }
    }
}

impl fmt::Display for InternationalMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternationalMode::On => write!(f, "INTL-ON"),
            InternationalMode::Off =>  write!(f, "INTL-OFF"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(u8)]
pub enum CacheMode {
    #[default]
    Off  = 0,
    On = 0x04,
}

impl FromStr for CacheMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "on"|"yes" => Ok(CacheMode::On),
            "off"|"no" => Ok(CacheMode::Off),
            _ => Err(Error::InvalidCacheModeError)
        }
    }
}

impl fmt::Display for CacheMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheMode::On => write!(f, "CACHE-ON"),
            CacheMode::Off =>  write!(f, "CACHE-OFF"),
        }
    }
}
