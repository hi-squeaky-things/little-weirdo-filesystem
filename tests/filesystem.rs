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
        assert_eq!(size_of_value, size as u32);
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
        let free_blocks = filesystem.amount_of_free_blocks();
        //        assert_eq!(free_blocks, 486);

        assert_eq!(size_of_value, payload.len() as u32);
        assert_eq!(payload, &buffer[..size_of_value as usize]);
    }

    #[test]
    fn test_size_of_key() {
        let storage = MemoryStorage::new(0x100000);
        let mut filesystem: WeirdoFileSystem<MemoryStorage> =
            WeirdoFileSystem::new(storage, 0, 0x100000);
        filesystem.format();

        let payload = include_bytes!("./mock-data/mock1.bin");
        let _result = filesystem.write_key_value(800, payload).unwrap();

        assert_eq!(
            filesystem.size_of_key_value(800).unwrap(),
            payload.len() as u32
        );
    }

    #[test]
    fn test_fs_chunking_larger_then_32kb() {
        let storage = MemoryStorage::new(0x100000);
        let mut filesystem: WeirdoFileSystem<MemoryStorage> =
            WeirdoFileSystem::new(storage, 0, 0x100000);
        filesystem.format();

        let payload = include_bytes!("./mock-data/mock2.bin");
        let _result = filesystem.write_key_value(800, payload).unwrap();

        let mut buffer: [u8; 300_000] = [0; 300_000];
        let size_of_value = filesystem.read_key_value(800, &mut buffer).unwrap();
        assert_eq!(size_of_value, payload.len() as u32);
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

    #[test]
    fn test_retrieve_elements() {
        let mut storage = MemoryStorage::new(0x100000);
        let mock_data = include_bytes!("./mock-data/filesystem_mock0.bin");
        storage.load(mock_data);

        let mut filesystem: WeirdoFileSystem<MemoryStorage> =
            WeirdoFileSystem::new(storage, 0, 0x100000);

        let mut buffer: [u8; 1200] = [0; 1200];
        let size_of_value = filesystem.read_key_value(700, &mut buffer);
        println!("size {:?}", size_of_value);
    }
}
