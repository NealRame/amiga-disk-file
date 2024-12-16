use std::time::Duration;

use crate::disk::BLOCK_SIZE;

// Time ///////////////////////////////////////////////////////////////////////
pub const AMIGA_EPOCH_OFFSET                 : Duration = Duration::from_secs(252460800);
pub const TICKS_PER_SECOND                   : u32 = 50;

// Block //////////////////////////////////////////////////////////////////////
pub const BLOCK_PRIMARY_TYPE_OFFSET          : usize = 0;

pub const BLOCK_SECONDARY_TYPE_OFFSET        : usize = BLOCK_SIZE - 4;

pub const BLOCK_CHECKSUM_OFFSET              : usize = 0x14;

pub const BLOCK_HASH_TABLE_OFFSET            : usize = 0x18;

// Boot block /////////////////////////////////////////////////////////////////
pub const BOOT_BLOCK_CHECKSUM_OFFSET         : usize = 0x04;

pub const BOOT_BLOCK_ROOT_BLOCK_OFFSET       : usize = 0x08;

pub const BOOT_BLOCK_BOOT_CODE_OFFSET        : usize = 0x0c;
pub const BOOT_BLOCK_BOOT_CODE_SIZE          : usize = 2*BLOCK_SIZE - 12;

pub const BOOT_BLOCK_DISK_TYPE_SLICE         : std::ops::Range<usize>
    = 0..BOOT_BLOCK_CHECKSUM_OFFSET;

pub const BOOT_BLOCK_CHECKSUM_SLICE          : std::ops::Range<usize>
    = BOOT_BLOCK_CHECKSUM_OFFSET..BOOT_BLOCK_ROOT_BLOCK_OFFSET;

pub const BOOT_BLOCK_ROOT_BLOCK_SLICE        : std::ops::Range<usize>
    = BOOT_BLOCK_ROOT_BLOCK_OFFSET..BOOT_BLOCK_BOOT_CODE_OFFSET;

pub const BOOT_BLOCK_BOOT_CODE_SLICE         : std::ops::Range<usize>
    = BOOT_BLOCK_BOOT_CODE_OFFSET..2*BLOCK_SIZE;

// Root block /////////////////////////////////////////////////////////////////
pub const ROOT_BLOCK_BITMAP_MAX_PAGES        : usize = 25;

pub const ROOT_BLOCK_DISK_NAME_MAX_SIZE      : usize = 30;

pub const ROOT_BLOCK_HASH_TABLE_SIZE         : usize = BLOCK_SIZE/4 - 56;

pub const ROOT_BLOCK_HASH_TABLE_SIZE_OFFSET  : usize = 0x0c;

pub const ROOT_BLOCK_BITMAP_FLAG_OFFSET      : usize = BLOCK_SIZE - 0xc8;
pub const ROOT_BLOCK_BITMAP_PAGES_OFFSET     : usize = BLOCK_SIZE - 0xc4;
pub const ROOT_BLOCK_BITMAP_EXTENSION_OFFSET : usize = BLOCK_SIZE - 0x60;

pub const ROOT_BLOCK_VOLUME_NAME_SIZE_OFFSET : usize = BLOCK_SIZE - 0x50;
pub const ROOT_BLOCK_VOLUME_NAME_OFFSET      : usize = BLOCK_SIZE - 0x4f;

pub const ROOT_BLOCK_R_DAYS_OFFSET           : usize = BLOCK_SIZE - 0x5c;
pub const ROOT_BLOCK_R_MINS_OFFSET           : usize = BLOCK_SIZE - 0x58;
pub const ROOT_BLOCK_R_TICKS_OFFSET          : usize = BLOCK_SIZE - 0x54;

pub const ROOT_BLOCK_V_DAYS_OFFSET           : usize = BLOCK_SIZE - 0x28;
pub const ROOT_BLOCK_V_MINS_OFFSET           : usize = BLOCK_SIZE - 0x24;
pub const ROOT_BLOCK_V_TICKS_OFFSET          : usize = BLOCK_SIZE - 0x20;

pub const ROOT_BLOCK_C_DAYS_OFFSET           : usize = BLOCK_SIZE - 0x1c;
pub const ROOT_BLOCK_C_MINS_OFFSET           : usize = BLOCK_SIZE - 0x18;
pub const ROOT_BLOCK_C_TICKS_OFFSET          : usize = BLOCK_SIZE - 0x14;

pub const ROOT_BLOCK_EXTENSION_OFFSET        : usize = BLOCK_SIZE - 0x08;
