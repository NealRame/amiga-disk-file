use std::ops::Rem;

use crate::disk::BLOCK_SIZE;
use crate::errors::*;

use super::InternationalMode;


fn valid_name_char(c: u8) -> bool {
    c >= b' ' && c != b':' && c != b'/' && c != 127
}

// Check if a (dir, file or volume) name is valid
pub fn check_name(name: &str) -> Result<(), Error> {
    if name.bytes().all(valid_name_char) {
        Ok(())
    } else {
        Err(Error::InvalidNameError)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_name_fails_with_an_invalid_name() {
        assert_eq!(check_name("foo:bar"), Err(Error::InvalidNameError));
        assert_eq!(check_name("foo/bar"), Err(Error::InvalidNameError));
    }

    #[test]
    fn check_name_pass_with_a_valid_name() {
        assert_eq!(check_name("foo"), Ok(()));
        assert_eq!(check_name("amiga"), Ok(()));
    }

    #[test]
    fn hash_name_is_ok() {
        assert_eq!(hash_name("foo", InternationalMode::Off), 15);
        assert_eq!(hash_name("bar", InternationalMode::Off), 24);
    }
}
