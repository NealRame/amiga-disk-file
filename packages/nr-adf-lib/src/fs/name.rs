use std::ops::Rem;

use crate::disk::BLOCK_SIZE;
use crate::errors::*;

use super::InternationalMode;


// Check if a (dir, file or volume) name is valid
pub fn check_name(name: &str) -> Result<(), Error> {
    for c in name.bytes() {
        if c < b' ' || c == b':' || c == b'/' {
            return Err(Error::InvalidNameError)
        }
    }
    Ok(())
}

// adapted from https://github.com/lclevy/ADFlib/blob/master/src/adf_dir.c#L918
fn to_upper(c: u8) -> u8 {
    if (0x61..=0x7a).contains(&c) {
        c - (0x61 - 0x41)
    } else {
        c
    }
}

// adapted from https://github.com/lclevy/ADFlib/blob/master/src/adf_dir.c#L912
fn to_upper_intl(c: u8) -> u8 {
    if (0x61..=0x7a).contains(&c)
    || (0xe0..=0xfe).contains(&c) && c != 0xf7 {
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
