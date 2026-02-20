extern crate alloc;
use alloc::vec::Vec;
use embedded_storage::{ReadStorage, Storage};

pub struct MemoryStorage {
    internal_memory: Vec<u8>,
}

#[derive(Debug)]
pub struct MemoryError;

impl MemoryStorage {
    pub fn new(size: usize) -> Self {
        MemoryStorage {
            internal_memory: alloc::vec![0; size],
        }
    }

    pub fn dump(&mut self) -> &[u8] {
        self.internal_memory.as_slice()
    }

    pub fn load(&mut self, payload: &[u8]) {
        let len = payload.len().min(self.internal_memory.len());
        self.internal_memory[..len].copy_from_slice(&payload[..len]);
    }
}

impl ReadStorage for MemoryStorage {
    type Error = MemoryError;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let start = offset as usize;
        let end = (offset + bytes.len() as u32) as usize;

        if end > self.internal_memory.len() {
            return Err(MemoryError);
        }

        bytes.copy_from_slice(&self.internal_memory[start..end]);
        Ok(())
    }

    fn capacity(&self) -> usize {
        return self.internal_memory.len();
    }
}

impl Storage for MemoryStorage {
    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        let start = offset as usize;
        let end = (offset + bytes.len() as u32) as usize;

        if end > self.internal_memory.len() {
            return Err(MemoryError);
        }

        self.internal_memory[start..end].copy_from_slice(bytes);
        Ok(())
    }
}
