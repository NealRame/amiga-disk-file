use crate::disk::BLOCK_SIZE;

pub const SECONDS_PER_DAY: u64 = 3600*24;
pub const SECONDS_PER_MINUTE: u64 = 60;

// Block //////////////////////////////////////////////////////////////////////
pub const BLOCK_CHECKSUM_OFFSET              : usize = 0x14;

// Boot block /////////////////////////////////////////////////////////////////
pub const BOOT_BLOCK_BOOT_CODE_SIZE          : usize = 2*BLOCK_SIZE - 12;

// Root block /////////////////////////////////////////////////////////////////
pub const ROOT_BLOCK_BITMAP_MAX_PAGES        : usize = 25;

pub const ROOT_BLOCK_DISK_NAME_MAX_SIZE      : usize = 30;

pub const ROOT_BLOCK_HASH_TABLE_MAX_SIZE     : usize = BLOCK_SIZE/4 - 56;

pub const ROOT_BLOCK_HASH_TABLE_SIZE_OFFSET  : usize = 0x0c;
pub const ROOT_BLOCK_HASH_TABLE_OFFSET       : usize = 0x18;

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
