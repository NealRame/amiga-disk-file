use std::cell::RefCell;
use std::rc::Rc;

use crate::block::*;
use crate::disk::*;
use crate::errors::*;

use super::amiga_dos::*;
use super::checksum::*;
use super::constants::*;

enum BitmapAction {
    Alloc,
    Free,
}

fn get_bitmap_block_count(
    disk: Rc<RefCell<Disk>>,
) -> usize {
    let block_count = disk.borrow().block_count() - 2; // 2 boot blocks

    match block_count%BITMAP_BLOCK_BIT_COUNT {
        0 => block_count/BITMAP_BLOCK_BIT_COUNT,
        _ => block_count/BITMAP_BLOCK_BIT_COUNT + 1,
    }
}

fn init_bitmap_block(
    disk: Rc<RefCell<Disk>>,
    bitmap_block_index: usize,
    root_block_address: LBAAddress,
) -> Result<(), Error> {
    let bitmap_block_address = root_block_address + 1 + bitmap_block_index;

    let mut root_block = Block::new(disk.clone(), root_block_address);
    root_block.write_u32(
        ROOT_BLOCK_BITMAP_PAGES_OFFSET + 4*bitmap_block_index,
        bitmap_block_address as u32,
    )?;

    // init the bitmap_block
    let mut bitmap_block = Block::new(disk.clone(), bitmap_block_address);
    bitmap_block.fill(0xff, 0, BLOCK_SIZE)?;

    Ok(())
}

fn init_bitmap_blocks(
    disk: Rc<RefCell<Disk>>,
    root_block_address: LBAAddress,
) -> Result<(), Error> {
    let bitmap_block_count = get_bitmap_block_count(disk.clone());

    for bitmap_block_index in 0..bitmap_block_count {
        init_bitmap_block(
            disk.clone(),
            bitmap_block_index,
            root_block_address,
        )?;
    }

    Ok(())
}

fn update_bitmap_blocks(
    disk: Rc<RefCell<Disk>>,
    bitmap_block_addresses: &[LBAAddress],
    action: BitmapAction,
    mut addr: LBAAddress,
) -> Result<(), Error> {
    let mut disk = disk.borrow_mut();
    addr = addr - 2;

    let page_index = addr/BITMAP_BLOCK_BIT_COUNT;


    let page = disk.blocks_mut(bitmap_block_addresses[page_index], 1)?;
    let page_bit_offset = 32 + addr%BITMAP_BLOCK_BIT_COUNT;

    let page_word_index = page_bit_offset/32;
    let page_word_offset = page_bit_offset%32;

    if let Some(dword) = page.chunks_mut(4).skip(page_word_index).next() {
        let byte_index = std::mem::size_of::<u32>() - 1 - page_word_offset/8;
        let bit_offset = page_word_offset%8;

        dword[byte_index] = match action {
            BitmapAction::Alloc => dword[byte_index] & !(1u8 << bit_offset),
            BitmapAction::Free => dword[byte_index] | (1u8 << bit_offset),
        };
    }

    let checksum = compute_checksum(page, 0);

    page[..4].copy_from_slice(&checksum.to_be_bytes());

    Ok(())
}

fn reserve_bitmap_blocks(
    disk: Rc<RefCell<Disk>>,
    root_block_address: LBAAddress,
) -> Result<(), Error> {
    let root_block = Block::new(disk.clone(), root_block_address);
    let bitmap_block_addresses = root_block.read_bitmap()?;

    for addr in [root_block_address].iter().chain(bitmap_block_addresses.iter()).copied() {
        update_bitmap_blocks(
            disk.clone(),
            &bitmap_block_addresses,
            BitmapAction::Alloc,
            addr as LBAAddress,
        )?;
    }
    Ok(())
}

fn find_free_block_address_in_bitmap_block(
    block_bitmap: &[u8],
    mut block_address_offset: LBAAddress,
) -> Option<LBAAddress> {
    for chunk in block_bitmap.chunks(4).skip(1) {
        let mut word = u32::from_be_bytes(chunk.try_into().unwrap());

        while word != 0 {
            if word & 0x01 != 0 {
                return Some(block_address_offset);
            }
            block_address_offset += 1;
            word = word >> 1;
        }
    }
    None
}

fn find_free_block_address(
    disk: Rc<RefCell<Disk>>,
    bitmap_block_addresses: &[LBAAddress],
) -> Result<Option<LBAAddress>, Error> {
    let mut block_address_offset = 2;
    let disk = disk.borrow(); // we need a ref

    for addr in bitmap_block_addresses {
        let block_bitmap = disk.blocks(*addr, 1)?;
        let block_address_free = find_free_block_address_in_bitmap_block(
            block_bitmap,
            block_address_offset,
        );

        if block_address_free.is_some() {
            return Ok(block_address_free);
        }

        block_address_offset += BITMAP_BLOCK_BIT_COUNT;
    }

    Ok(None)
}

/******************************************************************************
* BitmapInitializer ***********************************************************
******************************************************************************/

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
        disk: Rc<RefCell<Disk>>,
    ) -> Result<(), Error> {
        let root_block_address =
            self.root_block_address
                .unwrap_or_else(|| disk.borrow().block_count()/2);

        init_bitmap_blocks(disk.clone(), root_block_address)?;
        reserve_bitmap_blocks(disk.clone(), root_block_address)?;

        Ok(())
    }
}

/******************************************************************************
* AmigaDos ********************************************************************
******************************************************************************/

impl AmigaDosInner {
    pub fn reserve_block(
        &mut self,
    ) -> Result<LBAAddress, Error> {
        let bitmap_block_addresses = self.get_bitmap_block_addresses();

        match find_free_block_address(self.disk().clone(), &bitmap_block_addresses)? {
            Some(address) => {
                update_bitmap_blocks(
                    self.disk().clone(),
                    &bitmap_block_addresses,
                    BitmapAction::Alloc,
                    address,
                )?;
                Ok(address)
            },
            _ => Err(Error::NoSpaceLeft),
        }
    }

    pub fn free_block(
        &mut self,
        address: LBAAddress,
    ) -> Result<(), Error> {
        let bitmap_block_addresses = self.get_bitmap_block_addresses();

        update_bitmap_blocks(
            self.disk().clone(),
            &bitmap_block_addresses,
            BitmapAction::Free,
            address,
        )
    }
}
