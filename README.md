This repo contains a pure Rust implementation of `gunzip` (decompression only) from scratch in less than 1000 lines of code for educational purposes. You can check out branches 1 through 9 to understand the code in incremental steps. The current `main` branch is identical to branch 9.

The following roughly summarizes each stage
- branch 1: `main()` function and skeletal structure
- branch 2: `bitread` module for reading bits from a byte stream
- branch 3: `gzip` header & footer for parsing the metadata and checksum
- branch 4: `inflate` for block type 0 (uncompressed) data
- branch 5: `codebook` and `huffman_decoder` modules for decoding Huffman codes
- branch 6: `lz77` and `sliding_window` modules for decompressing LZ77-encoded data
- branch 7: `inflate` for block type 1 and 2 (compressed) data using fixed or dynamic Huffman codes
- branch 8: `checksum_write` module for verifying the decompressed data
- branch 9: performance optimization

# Build
```sh
$ cargo build -r
```

# Run
```sh
$ target/release/gunzip < compressed.gz > decompressed
```

# Contributing
You are welcome to contribute by submitting a PR for bug fixes or enhancements.

# Ports
You can also contribute by porting this code to a different language so that more people can learn from it.
