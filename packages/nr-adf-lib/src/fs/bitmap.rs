use crate::disk::*;
use crate::errors::*;

use super::block::*;
use super::checksum::compute_checksum;
use super::constants::*;


fn get_bitmap_block_count(
    disk: &Disk,
) -> usize {
    let block_count = disk.block_count() - 2; // 2 boot blocks

    match block_count%BITMAP_BLOCK_BIT_COUNT {
        0 => block_count/BITMAP_BLOCK_BIT_COUNT,
        _ => block_count/BITMAP_BLOCK_BIT_COUNT + 1,
    }
}

fn init_bitmap_block(
    disk: &mut Disk,
    bitmap_block_index: usize,
    root_block_address: LBAAddress,
) -> Result<(), Error> {
    let bitmap_block_address = root_block_address + 1 + bitmap_block_index;

    BlockWriter::try_from_disk(
        disk,
        root_block_address,
    )?.write_u32(
        ROOT_BLOCK_BITMAP_PAGES_OFFSET + 4*bitmap_block_index,
        bitmap_block_address as u32,
    )?;
    disk.block_mut(bitmap_block_address)?.fill(0xff);
    Ok(())
}

fn init_bitmap_blocks(
    disk: &mut Disk,
    root_block_address: LBAAddress,
) -> Result<(), Error> {
    let bitmap_block_count = get_bitmap_block_count(disk);

    for bitmap_block_index in 0..bitmap_block_count {
        init_bitmap_block(
            disk,
            bitmap_block_index,
            root_block_address,
        )?;
    }

    Ok(())
}

fn reserve_bitmap_block(
    disk: &mut Disk,
    addr: LBAAddress,
    bitmap_blocks: &[LBAAddress],
) -> Result<(), Error> {
    let page_index = (addr - 2)/BITMAP_BLOCK_BIT_COUNT;

    let page = disk.block_mut(bitmap_blocks[page_index])?;
    let page_bit_offset = 32 + (addr - 2)%BITMAP_BLOCK_BIT_COUNT;

    let page_word_index = page_bit_offset/32;
    let page_word_offset = page_bit_offset%32;

    if let Some(dword) = page.chunks_mut(4).skip(page_word_index).next() {
        let byte_index = 3 - page_word_offset/8;
        let bit_offset = page_word_offset%8;

        dword[byte_index] = dword[byte_index] & !(1u8 << bit_offset);
    }

    let checksum = compute_checksum(page, 0);

    page[..4].copy_from_slice(&checksum.to_be_bytes());

    Ok(())
}

fn reserve_bitmap_blocks(
    disk: &mut Disk,
    root_block_address: LBAAddress,
) -> Result<(), Error> {
    let root_block = BlockReader::try_from_disk(disk, root_block_address)?;
    let root_bitmap_page = root_block.read_bitmap()?;

    reserve_bitmap_block(disk, root_block_address, &root_bitmap_page)?;

    for addr in root_bitmap_page.iter() {
        reserve_bitmap_block(disk, *addr as LBAAddress, &root_bitmap_page)?;
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BitmapInitializer {
    root_block_address: Option<LBAAddress>,
}

impl BitmapInitializer {
    pub fn with_root_block_address(
        &mut self,
        addr: Option<LBAAddress>,
    ) -> &mut Self {
        self.root_block_address = addr;
        self
    }

    pub fn init(
        &self,
        disk: &mut Disk,
    ) -> Result<(), Error> {
        let root_block_address =
            self.root_block_address
                .unwrap_or_else(|| disk.block_count()/2);

        init_bitmap_blocks(disk, root_block_address)?;
        reserve_bitmap_blocks(disk, root_block_address)?;

        Ok(())
    }
}
