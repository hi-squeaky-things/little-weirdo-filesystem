#[cfg(test)]
mod unit_tests {
    extern crate alloc;
    use alloc::vec::Vec;

    use embedded_storage::Storage;
    use embedded_storage::ReadStorage;
    use little_weirdo_filesystem::memory_storage::MemoryStorage;

    #[test]
    fn test_memory_storage_read_write() {
        let mut storage = MemoryStorage::new(0x100000);

        // Write some data to the storage
        let write_data = b"Hello, world!";

        match storage.write(0, write_data) {
            Ok(_) => {}
            Err(e) => panic!("Write operation failed {:?}", e),
        }

        // Read the data back from the storage
        let mut read_buffer: Vec<u8> = alloc::vec![0; write_data.len()];
        match storage.read(0, &mut read_buffer) {
            Ok(_) => {
                assert_eq!(read_buffer.as_slice(), write_data);
            }
            Err(e) => panic!("Read operation failed {:?}", e),
        }
    }

    #[test]
    fn test_memory_storage_capacity() {
        let storage = MemoryStorage::new(0x100000);
        assert_eq!(storage.capacity(), 0x100000); // Ensure the data vector has the correct size
    }

    
}
