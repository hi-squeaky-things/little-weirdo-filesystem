#![no_std]
use embedded_storage::{self, Storage};
extern crate alloc;

pub mod memory_storage;

/// A simple filesystem implementation using embedded storage.
/// It manages key-value pairs stored in fixed-size blocks.
/// Each block can hold a portion of the value, with chaining for larger values.
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

// Block structure: 2048 bytes total
// [0] = 'E' (Empty) / 'O' (Occupied)
// [1..2] = key (u16)
// [3..4] = size of payload (u16, max 2041 if next_block exists)
// [5..6] = key of next_block in chain (u16, 0xFFFF if end)
// [7..] = payload (max 2041 bytes)

/// Status of a block in the filesystem.
pub enum BlockStatus {
    Empty = 'E' as isize,
    Occupied = 'O' as isize,
}

// Offsets within each block
const OFFSET_BLOCK_STATUS: u8 = 0x00;
const OFFSET_ADDRESS_KEY: u8 = 0x01;
const OFFSET_ADDRESS_SIZE: u8 = 0x05;
const OFFSET_ADDRESS_NEXT: u8 = 0x07;
const OFFSET_VALUE_SIZE: u8 = 0x0B;
const OFFSET_ADDRESS_PAYLOAD: u8 = 0x0B + 4;
const MAX_KEY_ID: u32 = 999;
pub const BLOCK_SIZE: u16 = 2048;

impl<T> WeirdoFileSystem<T>
where
    T: Storage,
{
    /// Creates a new WeirdoFileSystem instance.
    /// Initializes the filesystem with the given storage, offset, and size.
    /// Builds the internal cache of empty blocks.
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

    /// Formats the filesystem by marking all blocks as empty.
    /// Rebuilds the cache afterward.
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

    /// Builds the cache by scanning blocks to find the first empty block and total blocks.
    /// Updates empty_block and total_blocks fields.
    pub fn build_cache(&mut self) {
        self.empty_block = 0;
        self.total_blocks = self.size / self.block_size as u32;
        for block in 0..self.total_blocks {
            let address = self.offset + (block * self.block_size as u32);
            let mut block_status = [0u8; 1];
            let _ = self
                .storage
                .read(address + OFFSET_BLOCK_STATUS as u32, &mut block_status);
            if block_status[0] == BlockStatus::Empty as u8 {
                return;
            } else {
                self.empty_block += 1;
            }
        }
    }

    /// Returns the number of free (empty) blocks available.
    pub fn amount_of_used_blocks(&mut self) -> u32 {
        self.empty_block
    }

    /// Returns the number of free (empty) blocks available.
    pub fn amount_of_free_blocks(&mut self) -> u32 {
        self.total_blocks - self.empty_block
    }

    pub fn total_blocks(&mut self) -> u32 {
        self.total_blocks
    }

    pub fn size_of_key_value(&mut self, key: u32) -> Result<u32, WeirdoFileSystemError> {
        if key > MAX_KEY_ID {
            return Err(WeirdoFileSystemError::KeyToLarge);
        }
        let (found, found_block_address) = self.contains_key(key);
        let mut stored_size_of_value = [0u8; 4];
        if found {
            let _ = self.storage.read(
                found_block_address + OFFSET_VALUE_SIZE as u32,
                &mut stored_size_of_value,
            );
            return Ok(u32::from_le_bytes(stored_size_of_value));
        }
        Ok(0u32)
    }

    /// Writes a key-value pair to the filesystem.
    /// Splits the payload into chunks if necessary and chains blocks.
    /// Returns an error if the key is too large.
    /// TODO: Add out-of-space check.
    pub fn write_key_value(
        &mut self,
        key: u32,
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

        let mut block_key: u32 = key;
        for (i, block) in chunks.enumerate() {
            let empty_block_address = self.addres_of_empty_block();
            if i == 0 {
                let _ = self.storage.write(
                    empty_block_address + OFFSET_VALUE_SIZE as u32,
                    &(payload.len() as u32).to_le_bytes(),
                );
            }
            // Mark block as occupied
            let _ = self.storage.write(
                empty_block_address + OFFSET_BLOCK_STATUS as u32,
                &[BlockStatus::Occupied as u8],
            );
            // Write key
            let _ = self.storage.write(
                empty_block_address + OFFSET_ADDRESS_KEY as u32,
                &block_key.to_le_bytes(),
            );
            // Write size of this chunk
            let _ = self.storage.write(
                empty_block_address + OFFSET_ADDRESS_SIZE as u32,
                &(block.len() as u16).to_le_bytes(),
            );
            // Write next key or end marker
            if block.len() < self.payload_size as usize {
                let _ = self.storage.write(
                    empty_block_address + OFFSET_ADDRESS_NEXT as u32,
                    &[0xFF, 0xFF],
                );
            } else {
                block_key += 1000;
                let _ = self.storage.write(
                    empty_block_address + OFFSET_ADDRESS_NEXT as u32,
                    &block_key.to_le_bytes(),
                );
            }

            // Write payload
            let _ = self
                .storage
                .write(empty_block_address + OFFSET_ADDRESS_PAYLOAD as u32, block);
            self.empty_block += 1;
        }

        Ok(())
    }

    /// Reads a key-value pair from the filesystem.
    /// Reassembles the value from chained blocks.
    /// Returns the size of the read data or an error.
    pub fn read_key_value(
        &mut self,
        key: u32,
        value: &mut [u8],
    ) -> Result<u32, WeirdoFileSystemError> {
        if key > MAX_KEY_ID {
            return Err(WeirdoFileSystemError::KeyToLarge);
        }

        let mut block_address: u32;
        let (found, found_block_address) = self.contains_key(key);
        if found {
            let mut payload_size: u32 = 0;
            let mut stored_size = [0u8; 2];
            let mut next_key = [0u8; 4];
            block_address = found_block_address;
            loop {
                // Read size of this chunk
                let _ = self
                    .storage
                    .read(block_address + OFFSET_ADDRESS_SIZE as u32, &mut stored_size);
                // Read payload into value buffer
                let _ = self.storage.read(
                    block_address + OFFSET_ADDRESS_PAYLOAD as u32,
                    &mut value[payload_size as usize..],
                );
                // Read next key
                let _ = self
                    .storage
                    .read(block_address + OFFSET_ADDRESS_NEXT as u32, &mut next_key);

                payload_size += u16::from_le_bytes(stored_size) as u32;
                let next_key_retreived = u32::from_le_bytes(next_key);

                if next_key_retreived == 0xFFFF {
                    break;
                }

                let (found, found_block_address) = self.contains_key(next_key_retreived);
                block_address = found_block_address;
                if !found {
                    return Err(WeirdoFileSystemError::KeyNotFound);
                }
            }

            Ok(payload_size)
        } else {
            Err(WeirdoFileSystemError::KeyNotFound)
        }
    }

    /// Checks if a key exists in the filesystem.
    /// Returns (found, block_address) tuple.
    fn contains_key(&mut self, key: u32) -> (bool, u32) {
        for block in 0..self.empty_block {
            let block_address = self.offset + (block * self.block_size as u32);
            let mut stored_key = [0u8; 4];
            let _ = self
                .storage
                .read(block_address + OFFSET_ADDRESS_KEY as u32, &mut stored_key);
            if stored_key == { key }.to_le_bytes() {
                return (true, block_address);
            }
        }
        (false, 0)
    }

    /// Returns the address of the next empty block.
    fn addres_of_empty_block(&mut self) -> u32 {
        self.offset + self.empty_block * self.block_size as u32
    }
}
