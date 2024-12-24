use std::ops::Rem;

use crate::disk::BLOCK_SIZE;
use crate::errors::*;

use super::InternationalMode;


// Check if a (dir, file or volume) name is valid
pub fn check_name(name: String) -> Result<String, Error> {
    for c in name.bytes() {
        if c < ' ' as u8 || c == ':' as u8 || c == '/' as u8 {
            return Err(Error::InvalidNameError)
        }
    }
    Ok(name)
}

// adapted from https://github.com/lclevy/ADFlib/blob/master/src/adf_dir.c#L918
fn to_upper(c: u8) -> u8 {
    if c >= 0x61 && c <= 0x7a {
        c - (0x61 - 0x41)
    } else {
        c
    }
}

// adapted from https://github.com/lclevy/ADFlib/blob/master/src/adf_dir.c#L912
fn to_upper_intl(c: u8) -> u8 {
    if c >= 0x61 && c <= 0x7a || c >= 224 && c <= 254 && c != 247 {
        c - (0x61 - 0x41)
    } else {
        c
    }
}

pub fn hash_name(
    name: &str,
    international_mode: InternationalMode,
) -> usize {
    name.as_bytes()
        .iter()
        .copied()
        .map(|c| {
            match international_mode {
                InternationalMode::On  => to_upper_intl(c),
                InternationalMode::Off => to_upper(c),
            }
        })
        .fold(name.len(), |mut hash, c| {
            (hash, _) = hash.overflowing_mul(13);
            (hash, _) = hash.overflowing_add(c as usize);
            hash & 0x07ff
        })
        .rem(BLOCK_SIZE/4 - 56)
}
