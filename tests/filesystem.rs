#[cfg(test)]
mod unit_tests {
    use little_weirdo_filesystem::WeirdoFileSystem;
    use little_weirdo_filesystem::memory_storage::MemoryStorage;

    #[test]
    fn test_fs_storage_read_write() {
        let storage = MemoryStorage::new(0x100000);
        let mut filesystem: WeirdoFileSystem<MemoryStorage> =
            WeirdoFileSystem::new(storage, 0, 0x100000);
        filesystem.format();

        let payload = include_bytes!("./mock-data/mock0.bin");
        let size = payload.len();

        filesystem.write_key_value(1, payload).unwrap();

        let mut buffer: [u8; 2042] = [0; 2042];
        let size_of_value = filesystem.read_key_value(1, &mut buffer).unwrap();
        assert_eq!(size_of_value, size as u16);
        assert_eq!(&buffer[..size_of_value as usize], payload);
    }

    #[test]
    fn test_fs_free_blocks() {
        let storage = MemoryStorage::new(0x100000);
        let mut filesystem: WeirdoFileSystem<MemoryStorage> =
            WeirdoFileSystem::new(storage, 0, 0x100000);
        filesystem.format();

        let payload = include_bytes!("./mock-data/mock0.bin");
        filesystem.write_key_value(1, payload).unwrap();
        filesystem.write_key_value(2, payload).unwrap();
        filesystem.write_key_value(3, payload).unwrap();
        let free_blocks = filesystem.amount_of_free_blocks();
        assert_eq!(free_blocks, 509);
        filesystem.build_cache();
        let free_blocks = filesystem.amount_of_free_blocks();
        assert_eq!(free_blocks, 509);
    }

    #[test]
    fn test_fs_chunking() {
        let storage = MemoryStorage::new(0x100000);
        let mut filesystem: WeirdoFileSystem<MemoryStorage> =
            WeirdoFileSystem::new(storage, 0, 0x100000);
        filesystem.format();

        let payload = include_bytes!("./mock-data/mock1.bin");
        let _result = filesystem.write_key_value(800, payload).unwrap();
        let _result = filesystem.write_key_value(801, payload).unwrap();

        let mut buffer: [u8; 30_000] = [0; 30_000];
        let size_of_value = filesystem.read_key_value(800, &mut buffer).unwrap();
        assert_eq!(size_of_value, payload.len() as u16);
        assert_eq!(payload, &buffer[..size_of_value as usize]);
    }

    #[test]
    fn test_fs_format() {
        let storage = MemoryStorage::new(0x100000);
        let mut filesystem: WeirdoFileSystem<MemoryStorage> =
            WeirdoFileSystem::new(storage, 0, 0x100000);
        filesystem.format();

        let payload = include_bytes!("./mock-data/mock0.bin");
        filesystem.write_key_value(1, payload).unwrap();
        filesystem.write_key_value(2, payload).unwrap();
        filesystem.write_key_value(3, payload).unwrap();
        let free_blocks = filesystem.amount_of_free_blocks();
        assert_eq!(free_blocks, 509);
        filesystem.format();
        let free_blocks = filesystem.amount_of_free_blocks();
        assert_eq!(free_blocks, 512);
    }
}
