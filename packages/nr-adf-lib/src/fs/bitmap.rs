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

    Block::new(disk.clone(), root_block_address).write_checksum()
}

fn update_bitmap_blocks(
    disk: Rc<RefCell<Disk>>,
    bitmap_block_addresses: &[LBAAddress],
    action: BitmapAction,
    mut addr: LBAAddress,
) -> Result<(), Error> {
    let mut disk = disk.borrow_mut();

    if addr > disk.block_count() {
        return Err(Error::DiskInvalidLBAAddressError(addr));
    }

    addr -= 2;

    let page_index = addr/BITMAP_BLOCK_BIT_COUNT;
    let page = disk.blocks_mut(bitmap_block_addresses[page_index], 1)?;

    let page_bit_offset = addr%BITMAP_BLOCK_BIT_COUNT;
    let page_word_index = page_bit_offset/32;
    let page_word_offset = page_bit_offset%32;

    if let Some(dword) = page.chunks_mut(4).skip(1).nth(page_word_index) {
        let byte_index = std::mem::size_of::<u32>() - 1 - page_word_offset/8;
        let bit_offset = page_word_offset%8;

        dword[byte_index] = match action {
            BitmapAction::Alloc => dword[byte_index] & !(1u8 << bit_offset),
            BitmapAction::Free => dword[byte_index] | (1u8 << bit_offset),
        };
    }

    let checksum = compute_checksum(page, BITMAP_BLOCK_CHECKSUM_OFFSET);

    page[..4].copy_from_slice(&checksum.to_be_bytes());

    Ok(())
}

fn reserve_bitmap_blocks(
    disk: Rc<RefCell<Disk>>,
    root_block_address: LBAAddress,
) -> Result<(), Error> {
    let bitmap_block_addresses = Block::new(
        disk.clone(),
        root_block_address
    ).read_bitmap()?;

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

fn find_free_block_address_in_bitmap_chunk(
    bitmap_chunk: &[u8],
    mut bitmap_len: usize,
    mut block_address_offset: LBAAddress,
) -> Option<LBAAddress> {
    for chunk in bitmap_chunk.chunks(4) {
        let mut word = u32::from_be_bytes(chunk.try_into().unwrap());

        if word == 0 {
            bitmap_len -= 32;
            block_address_offset += 32;
        } else {
            while bitmap_len > 0 && word & 0x01 == 0 {
                bitmap_len -= 1;
                block_address_offset += 1;
                word >>= 1;
            }

            if bitmap_len != 0 {
                return Some(block_address_offset);
            }
        }
    }

    None
}

fn find_free_block_address(
    disk: Rc<RefCell<Disk>>,
    bitmap_block_addresses: &[LBAAddress],
) -> Result<Option<LBAAddress>, Error> {
    let disk_ref = disk.borrow();

    let disk_block_count = disk_ref.block_count();
    let mut address_offset = 2;

    for addr in bitmap_block_addresses {
        // bitmap length i.e. the count of blocks
        let bitmap_len =
            if address_offset + BITMAP_BLOCK_BIT_COUNT > disk_block_count {
                disk_block_count - address_offset
            } else {
                BITMAP_BLOCK_BIT_COUNT
            };

        // bitmap dword aligned length
        let bitmap_byte_len =
            if bitmap_len%32 > 0 {
                4*(bitmap_len/32 + 1)
            } else {
                bitmap_len/8
            };

        let block = disk_ref.blocks(*addr, 1)?;
        let bitmap = &block[4 .. 4 + bitmap_byte_len];

        let address = find_free_block_address_in_bitmap_chunk(
            bitmap,
            bitmap_len,
            address_offset,
        );

        if address.is_some() {
            return Ok(address);
        }

        address_offset += BITMAP_BLOCK_BIT_COUNT;
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
            self.root_block_address.unwrap_or_else(|| {
                disk.borrow().block_count()/2
            });

        init_bitmap_blocks(disk.clone(), root_block_address)?;
        reserve_bitmap_blocks(disk.clone(), root_block_address)?;

        Ok(())
    }
}

/******************************************************************************
* AmigaDos ********************************************************************
******************************************************************************/

impl AmigaDosInner {
    #[cfg(test)]
    pub fn get_bitmap(
        &self,
    ) -> Result<Vec<u8>, Error> {
        let bitmap_block_addresses = self.get_bitmap_block_addresses();
        let mut bitmap = vec![0u8; bitmap_block_addresses.len()*BLOCK_SIZE];

        for (i, addr) in bitmap_block_addresses.iter().copied().enumerate() {
            let slice = &mut bitmap[i*BLOCK_SIZE..(i + 1)*BLOCK_SIZE];

            Block::new(
                self.disk(),
                addr
            ).read_u8_array(0, slice)?;
        }

        Ok(bitmap)
    }
}

impl AmigaDosInner {
    pub fn reserve_block(
        &mut self,
    ) -> Result<LBAAddress, Error> {
        let bitmap_block_addresses = self.get_bitmap_block_addresses();


        match find_free_block_address(self.disk(), &bitmap_block_addresses)? {
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

    pub fn block_count(
        &self,
    ) -> usize {
        self.disk().borrow().block_count()
    }

    pub fn block_used(
        &self,
    ) -> Result<usize, Error> {
        unimplemented!()
    }
}
