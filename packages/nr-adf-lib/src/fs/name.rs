use crate::errors::*;


pub fn check_name(name: String) -> Result<String, Error> {
    for c in name.bytes() {
        if c < ' ' as u8 || c == ':' as u8 || c == '/' as u8 {
            return Err(Error::InvalidNameError)
        }
    }
    Ok(name)
}
