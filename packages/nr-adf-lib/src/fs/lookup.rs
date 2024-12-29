use std::path::Path;

use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::block::*;
use super::boot_block::*;


fn path_split<P: AsRef<Path>>(
    path: P,
) -> Option<Vec<String>> {
    path.as_ref().to_str()
        .map(|path| path.split("/"))
        .map(|strs| strs.filter_map(|s| {
            if s.len() > 0 {
                Some(String::from(s))
            } else {
                None
            }
        }))
        .map(|res| res.collect::<Vec<String>>())
}

impl AmigaDos {
    pub(super) fn lookup<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<LBAAddress, Error> {
        if let Some(path) = path_split(path) {
            let disk = self.disk();
            let boot_block = BootBlock::try_from_disk(disk)?;
            let international_mode = boot_block.get_international_mode();

            let mut block_addr = boot_block.get_root_block_address();

            for name in path {
                let br = BlockReader::try_from_disk(disk, block_addr)?;

                if let Some(addr) = br.lookup(&name, international_mode)? {
                    block_addr = addr;
                } else {
                    return Err(Error::NotFoundError);
                }
            }

            Ok(block_addr)
        } else {
            Err(Error::InvalidPathError)
        }
    }
}
