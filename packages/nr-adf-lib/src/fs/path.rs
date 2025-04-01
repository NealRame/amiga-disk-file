use std::ffi::OsStr;
use std::path::Path;

use crate::errors::*;


pub(super) fn split<P: AsRef<Path>>(
    path: P,
) -> Option<Vec<String>> {
    path.as_ref().to_str()
        .map(|path| path.split("/"))
        .map(|strs| strs.filter_map(|s| {
            if !s.is_empty() {
                Some(String::from(s))
            } else {
                None
            }
        }))
        .map(|res| res.collect::<Vec<String>>())
}

pub(super) fn get_basename(
    path: &Path,
) -> Result<&str, Error> {
    path.file_name()
        .and_then(OsStr::to_str)
        .ok_or(Error::InvalidPathError)
}

pub(super) fn get_dirname(
    path: &Path,
) -> Result<&Path, Error> {
    path.parent().ok_or(Error::InvalidPathError)
}
