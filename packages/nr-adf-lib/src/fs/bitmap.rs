use std::cell::RefCell;
use std::ops::Range;
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

#[derive(Clone, Debug)]
struct BitmapBlock {
    address: LBAAddress,
    address_range: Range<LBAAddress>,
    disk: Rc<RefCell<Disk>>,
}

impl BitmapBlock {
    fn byte_len(&self) -> usize {
        let block_count = self.block_count();

        // bitmap dword aligned length
        if block_count%32 > 0 {
            4*(block_count/32 + 1)
        } else {
            block_count/8
        }
    }

    fn block_count(&self) -> usize {
        self.address_range.len()
    }

    fn contains_block(
        &self,
        address: LBAAddress,
    ) -> bool {
        self.address_range.contains(&address)
    }

    fn count_free_block(
        &self,
    ) -> usize {
        let disk_ref = self.disk.borrow();

        let block = disk_ref.blocks(self.address, 1).unwrap();
        let bytes = &block[4 .. 4 + self.byte_len()];

        let mut remaining = self.block_count() as u32;

        bytes
            .chunks(4)
            .map(|chunk| u32::from_be_bytes(chunk.try_into().unwrap()))
            .map(|mut dword| {
                if remaining >= 32 {
                    remaining -= 32;
                    dword.count_ones() as usize
                } else {
                    dword &= 0xffff_ffffu32 >> (32 - remaining);
                    dword.count_ones()  as usize
                }
            })
            .sum::<usize>()
    }

    fn find_first_free_block(
        &self,
    ) -> Option<LBAAddress> {
        let disk_ref = self.disk.borrow();

        let block = disk_ref.blocks(self.address, 1).unwrap();
        let bytes = &block[4 .. 4 + self.byte_len()];

        let mut address_offset = self.address_range.start;
        let mut remaining = self.block_count();

        for chunk in bytes.chunks(4) {
            let mask = if remaining > 32 {
                0xffff_ffffu32
            } else {
                0xffff_ffffu32.checked_shl(32 - remaining as u32).unwrap_or(0)
            };

            let dword = u32::from_be_bytes(chunk.try_into().unwrap());

            if (dword & mask) == 0 {
                address_offset += 32.min(remaining);
                remaining -= 32.min(remaining);
            } else {
                for bit in 0..remaining.min(32) {
                    if dword & (1u32 << bit) != 0 {
                        return Some(address_offset + bit);
                    }
                }
            }
        }

        None
    }

    fn update_block(
        &mut self,
        mut addr: LBAAddress,
        action: BitmapAction,
    ) -> Result<(), Error> {
        let mut disk_ref = self.disk.borrow_mut();

        let block = disk_ref.blocks_mut(self.address, 1).unwrap();
        let bytes = &mut block[4 .. 4 + self.byte_len()];

        if !self.contains_block(addr) {
            return Err(Error::DiskInvalidLBAAddressError(addr));
        }

        addr -= 2;

        let bit_offset = addr%BITMAP_BLOCK_BIT_COUNT;

        let dword_index = bit_offset/32;
        let dword_bit = bit_offset%32;

        if let Some(chunk) = bytes.chunks_mut(4).nth(dword_index) {
            let mut dword = u32::from_be_bytes(chunk.try_into().unwrap());

            dword = match action {
                BitmapAction::Alloc => dword & !(1u32 << dword_bit),
                BitmapAction::Free => dword | (1u32 << dword_bit),
            };

            chunk.copy_from_slice(&dword.to_be_bytes());
        }

        let checksum = compute_checksum(block, BITMAP_BLOCK_CHECKSUM_OFFSET);

        block[..4].copy_from_slice(&checksum.to_be_bytes());

        Ok(())
    }

    fn try_reserve_block(
        &mut self,
    ) -> Option<LBAAddress> {
        if let Some(address) = self.find_first_free_block() {
            self.update_block(address, BitmapAction::Alloc).unwrap();
            Some(address)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
struct BitmapBlockIterator {
    disk: Rc<RefCell<Disk>>,
    bitmap_block_addresses: Vec<LBAAddress>,
    block_address_offset: LBAAddress,
}

impl BitmapBlockIterator {
    fn new(
        bitmap_block_addresses: &[LBAAddress],
        disk: Rc<RefCell<Disk>>,
    ) -> BitmapBlockIterator {
        BitmapBlockIterator {
            disk,
            bitmap_block_addresses: Vec::from(bitmap_block_addresses),
            block_address_offset: 2,
        }
    }
}

impl Iterator for BitmapBlockIterator {
    type Item = BitmapBlock;

    fn next(&mut self) -> Option<Self::Item> {
        let block_count = self.disk.borrow().block_count();

        if let Some(address) = self.bitmap_block_addresses.pop() {
            let first = self.block_address_offset;
            let last =
                if first + BITMAP_BLOCK_BIT_COUNT > block_count {
                    block_count
                } else {
                    BITMAP_BLOCK_BIT_COUNT
                };

            Some(BitmapBlock {
                address,
                address_range: first..last,
                disk: self.disk.clone(),
            })
        } else {
            None
        }
    }
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

        let mut reserved_blocks = vec![root_block_address];
        let mut bitmap_blocks = vec![];

        let bitmap_block_count = get_bitmap_block_count(disk.clone());
        for bitmap_block_index in 0..bitmap_block_count {
            let bitmap_block_address = root_block_address + 1 + bitmap_block_index;

            // init the bitmap_block
            Block::new(
                disk.clone(),
                bitmap_block_address
            ).fill(0xff, 0, BLOCK_SIZE)?;

            // write bitmap block address in the root block bitmap index table
            Block::new(
                disk.clone(),
                root_block_address
            ).write_u32(
                ROOT_BLOCK_BITMAP_PAGES_OFFSET + 4*bitmap_block_index,
                bitmap_block_address as u32,
            )?;

            reserved_blocks.push(bitmap_block_address);
            bitmap_blocks.push(bitmap_block_address);
        }

        Block::new(disk.clone(), root_block_address).write_checksum()?;

        // reserve root and bitmap blocks in the bitmap
        for address in reserved_blocks {
            BitmapBlockIterator::new(bitmap_blocks.as_slice(), disk.clone())
                .find(|bitmap_block| {
                    bitmap_block.contains_block(address)
                })
                .ok_or(Error::DiskInvalidLBAAddressError(address))
                .and_then(|mut bitmap_block| {
                    bitmap_block.update_block(address, BitmapAction::Alloc)
                })?;
        }

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
    fn bitmap_block_iter(
        &self,
    ) -> BitmapBlockIterator {
        BitmapBlockIterator::new(
            self.get_bitmap_block_addresses().as_slice(),
            self.disk(),
        )
    }

    pub fn reserve_block(
        &mut self,
    ) -> Result<LBAAddress, Error> {
        self.bitmap_block_iter()
            .find_map(|mut bitmap_block| bitmap_block.try_reserve_block())
            .ok_or(Error::NoSpaceLeft)
    }

    pub fn free_block(
        &mut self,
        address: LBAAddress,
    ) -> Result<(), Error> {
        self.bitmap_block_iter()
            .find(|bitmap_block| bitmap_block.contains_block(address))
            .ok_or(Error::DiskInvalidLBAAddressError(address))
            .and_then(|mut bitmap_block| {
                bitmap_block.update_block(address, BitmapAction::Free)
            })
    }

    pub fn total_block_count(
        &self,
    ) -> usize {
        self.disk().borrow().block_count()
    }

    pub fn free_block_count(
        &self,
    ) -> usize {
        self.bitmap_block_iter()
            .map(|bitmap_block| bitmap_block.count_free_block())
            .sum()
    }
}
