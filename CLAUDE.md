# rust-marisa Project Guidelines

## Project Goal

This project is a Rust port of [marisa-trie](https://github.com/s-yata/marisa-trie), a static and space-efficient trie data structure library originally written in C++.

The primary goal is to create a faithful Rust implementation that maintains compatibility with the original library's design and behavior, while leveraging Rust's safety features and idioms.

## Core Principles

### 1. Respect the Original Structure

- **Mirror the directory structure**: The Rust codebase should reflect the original C++ structure as closely as possible
  - `lib/marisa/` → `src/marisa/`
  - `lib/marisa/grimoire/` → `src/grimoire/`
  - `include/marisa/*.h` → public API modules in `src/lib.rs` or dedicated modules

- **Maintain logical organization**: Keep related functionality together as in the original
  - `grimoire/io/` → `grimoire/io/` (reader, writer, mapper)
  - `grimoire/trie/` → `grimoire/trie/` (louds-trie, tail, cache, etc.)
  - `grimoire/vector/` → `grimoire/vector/` (bit-vector, flat-vector, etc.)
  - `grimoire/algorithm/` → `grimoire/algorithm/` (sorting, etc.)

### 2. Track Source File Mapping

Each Rust module should clearly indicate which C++ file(s) it was ported from:

```rust
//! Ported from: lib/marisa/grimoire/trie/louds-trie.h
//! Ported from: lib/marisa/grimoire/trie/louds-trie.cc
//!
//! LOUDS (Level-Order Unary Degree Sequence) trie implementation.
```

For files that combine multiple C++ sources:

```rust
//! Ported from:
//! - include/marisa/trie.h
//! - lib/marisa/trie.cc
//!
//! Main trie interface.
```

### 3. Preserve Data Structures and Algorithms

- **Keep the same data structures**: Use Rust equivalents that maintain the same memory layout and behavior where possible
  - C++ `std::vector<T>` → `Vec<T>` or custom wrappers if specific behavior is needed
  - C++ bitfields and packed structures → Rust structs with appropriate field ordering
  - C++ templates → Rust generics

- **Maintain algorithmic approaches**: The core algorithms (LOUDS construction, search, etc.) should follow the same logic as the original

- **Preserve constants and magic numbers**: Keep the same configuration constants, thresholds, and tuning parameters used in the original

- **Document deviations**: When the Rust implementation must differ from C++ (e.g., for safety or idiomatic reasons), document why:

```rust
// Note: Original C++ uses raw pointer arithmetic here.
// Rust implementation uses safe indexing with the same logic.
```

### 4. Enable Tracking of Upstream Changes

- **Reference original commit hashes**: In CLAUDE.md or a PORTING.md file, record the marisa-trie commit that was used as the porting baseline

- **Mark porting status**: Track which files have been ported, which are in progress, and which are pending

- **Keep parallel structure**: Avoid premature refactoring that would make it difficult to diff against the original

### 5. Port Test Cases

- **Port all test files**: Tests from `tests/` directory should be ported to Rust tests
  - `tests/base-test.cc` → `tests/base_test.rs` or module tests
  - `tests/trie-test.cc` → `tests/trie_test.rs`
  - `tests/vector-test.cc` → `tests/vector_test.rs`
  - etc.

- **Maintain test coverage**: Ensure the Rust version has equivalent or better test coverage

- **Use Rust testing conventions**: Convert C++ test macros to Rust's `#[test]` and assertion macros

### 6. Language and Documentation

- **All documentation in English**: Module docs, function docs, comments, README, etc.

- **All commit messages in English**: Follow conventional commit style:
  - `feat: add louds-trie implementation`
  - `fix: correct bit-vector rank calculation`
  - `docs: add module documentation for grimoire::io`
  - `test: port trie-test.cc test cases`

- **Reference original when helpful**: In documentation, reference the original C++ implementation when it aids understanding

## Rust-Specific Considerations

### Safety and Idioms

- **Leverage Rust's type system**: Use `Option<T>` instead of null pointers, `Result<T, E>` for error handling

- **Use idiomatic Rust**: Follow Rust naming conventions (snake_case for functions/variables, CamelCase for types)

- **Minimize `unsafe`**: Only use `unsafe` where necessary for performance or C++ compatibility; document why it's needed

### API Design

- **Maintain C++ API semantics**: Public APIs should behave similarly to the original

- **Add Rust conveniences**: Implement traits like `Default`, `Clone`, `Debug` where appropriate

- **Iterator support**: Where C++ uses callback-based iteration, provide Rust iterators

## File Naming Convention

- C++ `.cc` files → Rust `.rs` files
- C++ `.h` header files → Rust modules or inline in `.rs` files
- Hyphenated names → snake_case: `louds-trie.cc` → `louds_trie.rs`

## Example Structure

```
rust-marisa/
├── src/
│   ├── lib.rs              # Main library entry (from include/marisa.h)
│   ├── agent.rs            # From lib/marisa/agent.{h,cc}
│   ├── keyset.rs           # From lib/marisa/keyset.{h,cc}
│   ├── trie.rs             # From lib/marisa/trie.{h,cc}
│   ├── grimoire/
│   │   ├── mod.rs
│   │   ├── io/
│   │   │   ├── mod.rs
│   │   │   ├── mapper.rs   # From lib/marisa/grimoire/io/mapper.{h,cc}
│   │   │   ├── reader.rs   # From lib/marisa/grimoire/io/reader.{h,cc}
│   │   │   └── writer.rs   # From lib/marisa/grimoire/io/writer.{h,cc}
│   │   ├── trie/
│   │   │   ├── mod.rs
│   │   │   ├── louds_trie.rs   # From lib/marisa/grimoire/trie/louds-trie.{h,cc}
│   │   │   ├── tail.rs         # From lib/marisa/grimoire/trie/tail.{h,cc}
│   │   │   └── ...
│   │   └── vector/
│   │       ├── mod.rs
│   │       ├── bit_vector.rs   # From lib/marisa/grimoire/vector/bit-vector.{h,cc}
│   │       └── ...
├── tests/
│   ├── base_test.rs        # From tests/base-test.cc
│   ├── trie_test.rs        # From tests/trie-test.cc
│   └── ...
├── benches/                # Optional: benchmarks (from tools/marisa-benchmark.cc)
└── examples/               # Optional: port tools as examples

```

## Porting Workflow

1. **Read the original file(s)** to understand the implementation
2. **Create corresponding Rust file(s)** with proper source attribution in module docs
3. **Port data structures** maintaining the same layout and semantics
4. **Port algorithms** following the same logic flow
5. **Port tests** to verify correctness
6. **Document deviations** where Rust idioms differ from C++
7. **Run tests** to ensure compatibility
8. **Update porting status** in tracking document

## References

- Original repository: https://github.com/s-yata/marisa-trie
- Baseline version: 0.3.1
- Baseline commit: `4ef33cc5a2b6b4f5e147e4564a5236e163d67982`
- Original license: BSD-2-Clause OR LGPL-2.1-or-later
