
#![no_std]
use embedded_storage::{self, Storage};
extern crate alloc;

pub mod memory_storage;

pub struct WeirdoFileSystem<T>
where
    T: Storage,
{
    pub storage: T,
    offset: u32,
    size: u32,
    empty_block: u32,
    total_blocks: u32,
    block_size: u16,
    payload_size: u16,
}

#[derive(Debug)]
pub enum WeirdoFileSystemError {
    PayloadTooLarge,
    KeyNotFound,
    KeyToLarge,
}

// block = 2048 = [[u8=status][u16=key][u16=size][u16=next_block][data]]
// [0] = 'E' (Empty) / 'O' (Occupied)
// [1..2] = key
// [3..4] = size of payload (size = 2041 if key next_block exists)
// [5..6] = key of next_block in chain
// [7..] = payload (max = 2048-7 = 2041 bytes)

pub enum BlockStatus {
    Empty = 'E' as isize,
    Occupied = 'O' as isize,
}

const OFFSET_BLOCK_STATUS: u8 = 0x00;
const OFFSET_ADDRESS_KEY: u8 = 0x01;
const OFFSET_ADDRESS_SIZE: u8 = 0x03;
const OFFSET_ADDRESS_NEXT: u8 = 0x05;
const OFFSET_ADDRESS_PAYLOAD: u8 = 0x07;
const MAX_KEY_ID: u16 = 999;
const BLOCK_SIZE: u16 = 2048;

impl<T> WeirdoFileSystem<T>
where
    T: Storage,
{
    pub fn new(storage: T, offset: u32, size: u32) -> Self {
        let mut new_fs = WeirdoFileSystem {
            storage,
            offset,
            size,
            empty_block: 0,
            total_blocks: 0,
            block_size: BLOCK_SIZE,
            payload_size: BLOCK_SIZE - OFFSET_ADDRESS_PAYLOAD as u16,
        };
        new_fs.build_cache();
        new_fs
    }

    pub fn format(&mut self) {
        for block in 0..self.total_blocks {
            let block_address = self.offset + (block * self.block_size as u32);
            let _ = self.storage.write(
                block_address + OFFSET_BLOCK_STATUS as u32,
                &[BlockStatus::Empty as u8],
            );
        }
        self.build_cache();
    }

    pub fn build_cache(&mut self) {
        self.empty_block = 0;
        self.total_blocks = self.size as u32 / self.block_size as u32;
        for block in 0..self.total_blocks {
            let address = self.offset + (block * self.block_size as u32);
            let mut block_status = [0u8; 1];
            let _ = self
                .storage
                .read(address + OFFSET_BLOCK_STATUS as u32, &mut block_status);
            if block_status[0] == BlockStatus::Empty as u8 {
                return;
            } else {
                self.empty_block = self.empty_block + 1;
            }
        }
    }

    pub fn amount_of_free_blocks(&mut self) -> u32 {
        self.total_blocks - self.empty_block
    }

    pub fn write_key_value(
        &mut self,
        key: u16,
        payload: &[u8],
    ) -> Result<(), WeirdoFileSystemError> {
        if key > MAX_KEY_ID {
            return Err(WeirdoFileSystemError::KeyToLarge);
        }
        // TODO Add a out-of-space check and throw an error if the value doesn't fit.
        //  if payload.len() + OFFSET_ADDRESS_PAYLOAD as usize >= self.block_size as usize {
        //      return Err(WeirdoFileSystemError::PayloadTooLarge);
        //  }

        let chunks = payload.chunks(self.payload_size as usize);

        let mut block_key = key;
        for (_i, block) in chunks.enumerate() {
            let empty_block_address = self.addres_of_empty_block();
            let _ = self.storage.write(
                empty_block_address + OFFSET_BLOCK_STATUS as u32,
                &[BlockStatus::Occupied as u8],
            );
            let _ = self.storage.write(
                empty_block_address + OFFSET_ADDRESS_KEY as u32,
                &block_key.to_le_bytes(),
            );
            let _ = self.storage.write(
                empty_block_address + OFFSET_ADDRESS_SIZE as u32,
                &(block.len() as u16).to_le_bytes(),
            );
            if block.len() < self.payload_size as usize {
                let _ = self
                    .storage
                    .write(empty_block_address + OFFSET_ADDRESS_NEXT as u32, &[0xFF, 0xFF]);
            } else {
                block_key = block_key + 1000;
                let _ = self.storage.write(
                    empty_block_address + OFFSET_ADDRESS_NEXT as u32,
                    &block_key.to_le_bytes(),
                );
            }

            let _ = self
                .storage
                .write(empty_block_address + OFFSET_ADDRESS_PAYLOAD as u32, block);
            self.empty_block = self.empty_block + 1;
        }

        Ok(())
    }

    pub fn read_key_value(
        &mut self,
        key: u16,
        value: &mut [u8],
    ) -> Result<u16, WeirdoFileSystemError> {
        if key > MAX_KEY_ID {
            return Err(WeirdoFileSystemError::KeyToLarge);
        }

        let mut block_address: u32;
        let (found, found_block_address) = self.contains_key(key);
        if found {
            let mut payload_size: u16 = 0;
            let mut stored_size = [0u8; 2];
            let mut next_key = [0u8; 2];
            block_address = found_block_address;
            loop {
                let _ = self
                    .storage
                    .read(block_address + OFFSET_ADDRESS_SIZE as u32, &mut stored_size);
                let _ = self.storage.read(
                    block_address + OFFSET_ADDRESS_PAYLOAD as u32,
                    &mut value[payload_size as usize..],
                );
                let _ = self
                    .storage
                    .read(block_address + OFFSET_ADDRESS_NEXT as u32, &mut next_key);

                payload_size += u16::from_le_bytes(stored_size);
                let next_key_retreived = u16::from_le_bytes(next_key);

                if next_key_retreived == 0xFFFF {
                    break;
                }

                let (found, found_block_address) = self.contains_key(next_key_retreived);
                block_address = found_block_address;
                if !found {
                    return Err(WeirdoFileSystemError::KeyNotFound);
                }
            }

            return Ok(payload_size);
        } else {
            Err(WeirdoFileSystemError::KeyNotFound)
        }
    }

    fn contains_key(&mut self, key: u16) -> (bool, u32) {
        for block in 0..self.empty_block {
            let block_address = self.offset + (block * self.block_size as u32);
            let mut stored_key = [0u8; 2];
            let _ = self
                .storage
                .read(block_address + OFFSET_ADDRESS_KEY as u32, &mut stored_key);
            if stored_key == key.to_le_bytes() {
                return (true, block_address);
            }
        }
        return (false, 0);
    }

    fn addres_of_empty_block(&mut self) -> u32 {
        return self.offset + self.empty_block * self.block_size as u32;
    }
}

