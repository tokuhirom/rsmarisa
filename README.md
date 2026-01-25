# rust-marisa

Rust port of [marisa-trie](https://github.com/s-yata/marisa-trie), a static and space-efficient trie data structure.

## About

MARISA (Matching Algorithm with Recursively Implemented StorAge) is a static and space-efficient trie data structure. This Rust implementation aims to maintain compatibility with the original C++ implementation while leveraging Rust's safety features.

## Features

A MARISA-based dictionary supports:
- **Lookup**: Check whether a given string exists in the dictionary
- **Reverse lookup**: Restore a key from its ID
- **Common prefix search**: Find keys from prefixes of a given string
- **Predictive search**: Find keys starting with a given string

## Status

🚧 **Work in Progress** - Active development in progress.

### Implemented

- ✅ **RankIndex**: Bit-packed rank storage for efficient rank queries
- ✅ **Vector<T>**: Generic container with serialization support
- ✅ **popcount**: Hardware-accelerated bit counting
- ✅ **BitVector**: Complete implementation with:
  - Basic operations: `push_back()`, `get()`, `size()`, `clear()`, `swap()`
  - Rank operations: `rank0()`, `rank1()` with O(1) complexity
  - Select operations: `select0()`, `select1()` with O(log n) complexity
  - Index building: `build()` with rank and select index construction
- ✅ **FlatVector**: Space-efficient integer vector with bit-packing
  - Automatically uses minimum bits based on maximum value
  - Example: 0-15 range uses 4 bits per value instead of 32

### Testing

- 175 comprehensive tests covering all implemented functionality
- All tests passing ✅
- Platform-specific tests for 32-bit and 64-bit systems

## Original Project

- Original repository: https://github.com/s-yata/marisa-trie
- Original author: Susumu Yata
- Baseline version: 0.3.1
- Baseline commit: `4ef33cc5a2b6b4f5e147e4564a5236e163d67982`

## License

BSD-2-Clause (same as the original project)

See [LICENSE](LICENSE) for details.

## Contributing

See [CLAUDE.md](CLAUDE.md) for porting guidelines and project structure.

