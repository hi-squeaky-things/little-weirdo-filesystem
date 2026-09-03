```
 ⡇ ⡇⢹⠁⢹⠁⡇ ⣏⡉ ⡇⢸⣏⡉⡇⣏⡱⡏⢱⡎⢱
 ⠧⠤⠇⠸ ⠸ ⠧⠤⠧⠤ ⠟⠻⠧⠤⠇⠇⠱⠧⠜⠣⠜
 filesystem
```

A Rust #no-std simple filesystem for embedded devices.

 See the Little Weirdo in embedded context action, checkout **[Little Squeaky Machine Hardware!](https://github.com/hi-squeaky-things/little-squeaky-machine-hardware)** or buy the embedded reference hardware @ **[Hi Squeaky Things](https://www.hi-squeaky-things.nl)** to support the development of this library.

<img src="https://www.hi-squeaky-things.nl/web/image/1400-5c54f567/Jouw%20alineatekst%20%281%29.webp" width="300">

> [!CAUTION]
> This project is actively being developed with frequent breaking changes. APIs may shift, features are incomplete, and stability is not guaranteed. Use at your own risk and expect regular updates that might require code adjustments. Have fun!

> [!IMPORTANT]
> **Hi Squeaky Things** can happen at any time. _Little Weirdo_ is ready to squeak, squuuueak, squeeeeeaak, squeaaaaaaaaak!

## Description

Little Weirdo Filesystem is a lightweight, no-std compatible filesystem designed for embedded systems. It uses embedded storage to manage key-value pairs in fixed-size blocks, supporting chaining for larger values. Keys are limited to u16 values up to 999, and payloads are split across blocks as needed.

## How to use it

Add the crate to your Cargo.toml:
```toml
[dependencies]
little-weirdo-filesystem = "0.1.0" 
embedded-storage = "0.3"  # Ensure you have embedded-storage
```

Import and use the filesystem:

```rust
use little_weirdo_filesystem::WeirdoFileSystem;
use embedded_storage::Storage;  // Assuming you have a storage implementation

// Assume [storage](http://_vscodecontentref_/0) is an instance of a type implementing Storage
let mut fs = WeirdoFileSystem::new(storage, offset, size);

// Format the filesystem (optional, but recommended for new storage)
fs.format();

// Write a key-value pair
let key = 42u16;
let value = b"Hello, Weirdo!";
fs.write_key_value(key, value).unwrap();

// Read the value back
let mut buffer = [0u8; 1024];  // Sufficiently large buffer
let size = fs.read_key_value(key, &mut buffer).unwrap();
let read_value = &buffer[..size as usize];
assert_eq!(read_value, value);
```

## Performance

Performance depends on the underlying storage. Reads and writes involve multiple storage operations for chained blocks. No benchmarks available yet.

## Credits

- [Small Braille ASCII Font](https://patorjk.com/software/taag/#p=display&f=Small+Braille&t=LITTLE+WEIRDO&x=rainbow1&v=1&h=1&w=80&we=false)


