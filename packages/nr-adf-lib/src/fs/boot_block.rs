use crate::disk::*;
use crate::errors::*;

use super::constants::*;
use super::options::*;


fn compute_checksum(data: &[u8]) -> u32 {
    const CHECKSUM_CHUNK_SIZE: usize = size_of::<u32>();

    let skip_offset = BOOT_BLOCK_CHECKSUM_OFFSET/CHECKSUM_CHUNK_SIZE;
    let mut checksum = 0u32;

    for (i, chunk) in data.chunks(4).enumerate() {
        if chunk.len() == 4 {
            let v = if i != skip_offset {
                u32::from_be_bytes(chunk.try_into().unwrap())
            } else {
                0
            };

            if u32::MAX < v {
                (checksum, _) = checksum.overflowing_add(1);
            }

            (checksum, _) = checksum.overflowing_add(v);
        }
    }

    !checksum
}

#[derive(Clone, Copy, Debug)]
pub struct BootBlockReader<'disk> {
    data: &'disk [u8],
}

impl<'disk> BootBlockReader<'disk> {
    pub fn try_from_disk(disk: &'disk Disk) -> Result<Self, Error> {
        let data = disk.blocks(0, 2)?;

        if &data[0..3] != &[0x44, 0x4f, 0x53] { // DOS
            return Err(Error::CorruptedImageFile);
        }

        let checksum = compute_checksum(&data);
        let expected = u32::from_be_bytes(data[4..8].try_into().unwrap());

        if checksum != expected {
            return Err(Error::CorruptedImageFile);
        }

        Ok(Self { data })
    }
}

impl BootBlockReader<'_> {
    pub fn get_filesystem_type(&self) -> FilesystemType {
        let flags = self.data[3];

        if flags & (FilesystemType::FFS as u8) != 0 {
            FilesystemType::FFS
        } else {
            FilesystemType::OFS
        }
    }

    pub fn get_international_mode(&self) -> InternationalMode {
        let flags = self.data[3];

        if flags & (InternationalMode::On as u8) != 0 {
            InternationalMode::On
        } else {
            InternationalMode::Off
        }
    }

    pub fn get_cache_mode(&self) -> CacheMode {
        let flags = self.data[3];

        if flags & (CacheMode::On as u8) != 0 {
            CacheMode::On
        } else {
            CacheMode::Off
        }
    }

    pub fn get_root_block_address(&self) -> LBAAddress {
        u32::from_be_bytes(
            self.data[BOOT_BLOCK_ROOT_BLOCK_SLICE].try_into().unwrap()
        ) as usize
    }

    pub fn get_boot_code(&self) -> &[u8] {
        &self.data[BOOT_BLOCK_BOOT_CODE_SLICE]
    }

    pub fn get_checksum(&self) -> u32 {
        u32::from_be_bytes(
            self.data[BOOT_BLOCK_CHECKSUM_SLICE].try_into().unwrap()
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BootBlockInitializer {
    boot_code: [u8; BOOT_BLOCK_BOOT_CODE_SIZE],
    root_block_address: Option<LBAAddress>,
    filesystem_type: FilesystemType,
    international_mode: InternationalMode,
    cache_mode: CacheMode,
}

impl Default for BootBlockInitializer {
    fn default() -> Self {
        return Self {
            boot_code: [0u8; BOOT_BLOCK_BOOT_CODE_SIZE],
            root_block_address: None,
            filesystem_type: FilesystemType::OFS,
            cache_mode: CacheMode::Off,
            international_mode: InternationalMode::Off,
        }
    }
}

impl BootBlockInitializer {
    // pub fn with_boot_code(
    //     &mut self,
    //     boot_code: &[u8; BOOT_BLOCK_BOOT_CODE_SIZE],
    // ) -> &mut Self {
    //     self.boot_code.copy_from_slice(boot_code);
    //     self
    // }

    pub fn with_root_block_address(
        &mut self,
        addr: Option<LBAAddress>,
    ) -> &mut Self {
        self.root_block_address = addr;
        self
    }

    pub fn with_filesystem_type(
        &mut self,
        filesystem_type: FilesystemType,
    ) -> &mut Self {
        self.filesystem_type = filesystem_type;
        self
    }

    pub fn with_international_mode(
        &mut self,
        international_mode: InternationalMode,
    )-> &mut Self {
        self.international_mode = international_mode;
        self
    }

    pub fn with_cache_mode(
        &mut self,
        cache_mode: CacheMode,
    ) -> &mut Self {
        self.cache_mode = cache_mode;
        self
    }

    pub fn init(&self, disk: &mut Disk) -> Result<(), Error> {
        let root_block_address =
            self.root_block_address.unwrap_or_else(|| disk.block_count()/2) as u32;

        let data = disk.blocks_mut(0, 2)?;

        let flags: u8 =
            self.cache_mode as u8
            | self.filesystem_type as u8
            | self.international_mode as u8;

        data[BOOT_BLOCK_DISK_TYPE_SLICE].copy_from_slice(
            &[0x44, 0x4f, 0x53, flags],
        );
        data[BOOT_BLOCK_ROOT_BLOCK_SLICE].copy_from_slice(
            &root_block_address.to_be_bytes(),
        );
        data[BOOT_BLOCK_BOOT_CODE_SLICE].copy_from_slice(
            &self.boot_code,
        );

        let checksum = compute_checksum(&data);

        data[BOOT_BLOCK_CHECKSUM_SLICE].copy_from_slice(
            &checksum.to_be_bytes(),
        );

        Ok(())
    }
}
