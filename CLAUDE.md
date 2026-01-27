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

### 4. Binary File Format Compatibility

**Critical Requirement**: Generated trie files must be binary-compatible with C++ marisa-trie.

- **Same serialization format**: The Rust implementation must produce files that can be read by C++ marisa-trie, and vice versa

- **Byte-level compatibility**: Ensure:
  - Same byte ordering (endianness handling)
  - Same padding and alignment
  - Same bit-packing schemes
  - Same integer sizes (use explicit types like `u32`, `u64` instead of `usize`)

- **Verification approach**:
  ```bash
  # Build trie with Rust implementation
  echo -e "app\napple\napricot" | cargo run --example marisa-build > rust.dic

  # Verify with C++ implementation
  echo "app" | marisa-lookup cpp.dic  # Should work
  echo "app" | marisa-lookup rust.dic # Should also work
  ```

- **Test cross-compatibility**: Create tests that verify:
  - Files created by C++ can be loaded by Rust
  - Files created by Rust can be loaded by C++
  - Both produce identical search results

### 5. Enable Tracking of Upstream Changes

- **Reference original commit hashes**: In CLAUDE.md or a PORTING.md file, record the marisa-trie commit that was used as the porting baseline

- **Mark porting status**: Track which files have been ported, which are in progress, and which are pending

- **Keep parallel structure**: Avoid premature refactoring that would make it difficult to diff against the original

### 6. Port Test Cases and Distinguish Origins

- **Port all test files**: Tests from `tests/` directory should be ported to Rust tests
  - `tests/base-test.cc` → `tests/base_test.rs` or module tests
  - `tests/trie-test.cc` → `tests/trie_test.rs`
  - `tests/vector-test.cc` → `tests/vector_test.rs`
  - etc.

- **Maintain test coverage**: Ensure the Rust version has equivalent or better test coverage

- **Use Rust testing conventions**: Convert C++ test macros to Rust's `#[test]` and assertion macros

- **Clearly distinguish test origins**: Tests must be clearly marked to indicate whether they are:
  1. **Ported from C++ original**: Tests directly ported from the C++ test suite
  2. **Rust-specific additions**: New tests added in the Rust version

**Marking Convention**:

```rust
// For tests ported from C++ marisa-trie:
#[test]
fn test_bit_vector_basic() {
    // Ported from: tests/vector-test.cc::TestBitVector
    let mut bv = BitVector::new();
    // ...
}

// For Rust-specific tests:
#[test]
fn test_bit_vector_rust_specific() {
    // Rust-specific: Test trait implementations
    let bv = BitVector::default();
    assert!(bv.is_empty());
}
```

**Why this is important**:
- Maintains traceability to original test coverage
- Helps identify compatibility requirements vs. Rust enhancements
- Enables verification against C++ behavior when debugging
- Ensures binary file format compatibility is properly tested

### 7. Language and Documentation

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

### Memory Safety with Raw Pointers

**Critical**: This codebase uses raw pointers extensively (especially in `Key` struct) to maintain C++ API compatibility. Extra care is required to avoid use-after-free bugs.

**Common Pitfall - Dangling Pointers**:
```rust
// ❌ WRONG: Temporary Vec is freed, leaving dangling pointer
let temp_vec = state.key_buf().to_vec();
agent.set_key_bytes(&temp_vec);  // Stores raw pointer
// temp_vec dropped here → pointer now invalid!

// ✅ CORRECT: Point to long-lived buffer
agent.set_key_from_state_buf();  // Points to agent's own state buffer
```

**Guidelines for Raw Pointer Usage**:

1. **Never point to temporary data**: Raw pointers must only reference data that will outlive the pointer's usage
   - ✅ Good: Point to agent's internal buffers (query, state.key_buf)
   - ✅ Good: Point to static data or heap-allocated data with known lifetime
   - ❌ Bad: Point to local Vec that gets dropped
   - ❌ Bad: Point to `.to_vec()` results

2. **Document lifetime assumptions**: When storing raw pointers, document the expected lifetime
   ```rust
   pub fn set_bytes(&mut self, bytes: &[u8]) {
       // SAFETY: Caller must ensure bytes outlive this Key instance
       self.ptr = Some(bytes.as_ptr());
   }
   ```

3. **Provide safe helper methods**: Add convenience methods that manage lifetimes correctly
   ```rust
   pub fn set_key_from_state_buf(&mut self) {
       // Safe: state is owned by self, so buffer lives as long as self
       let buf = self.state.as_ref().unwrap().key_buf();
       self.key.set_bytes(buf);
   }
   ```

4. **Test with Address Sanitizer**: When debugging memory issues, use:
   ```bash
   RUSTFLAGS="-Z sanitizer=address" cargo test
   ```

**Known Safe Patterns in this Codebase**:
- `agent.set_key_from_query()` - Points to agent's query buffer ✅
- `agent.set_key_from_state_buf()` - Points to agent's state buffer ✅
- `key.set_str(s)` where `s` is from function parameter - Caller ensures lifetime ✅

### API Design

- **Maintain C++ API semantics**: Public APIs should behave similarly to the original

- **Add Rust conveniences**: Implement traits like `Default`, `Clone`, `Debug` where appropriate

- **Iterator support**: Where C++ uses callback-based iteration, provide Rust iterators

### CLI Tools

**Naming Convention**: Use `rsmarisa-` prefix to avoid conflicts with C++ tools
  - `marisa-build` → `rsmarisa-build`
  - `marisa-lookup` → `rsmarisa-lookup`
  - etc.

**Testing Strategy**:
1. **Build release binaries first**: CLI tools should be tested in release mode
   ```bash
   cargo build --release --bin rsmarisa-build
   ```

2. **Create integration tests**: Verify output matches C++ tools exactly
   ```rust
   #[test]
   fn test_rsmarisa_lookup_compatibility() {
       // Build dict with C++ tool
       // Query with both Rust and C++ tools
       // Assert outputs are identical
   }
   ```

3. **Test all flags and options**: Ensure all command-line flags work correctly
   - Input/output formats (text, binary)
   - Configuration options (num_tries, cache_level, etc.)
   - Edge cases (empty input, large files, etc.)

4. **Verify with CI**: Add CLI tool tests to continuous integration

**Implementation Guidelines**:
- Use `clap` for argument parsing (consistent with modern Rust practices)
- Handle I/O errors properly with meaningful error messages
- Exit codes should match C++ tools where applicable
- Support stdin/stdout for pipeline usage

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

## Development Workflow

### Branch Protection

**IMPORTANT**: The `main` branch is protected. Direct pushes to `main` are not allowed.

**Required workflow**:
1. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**: Commit your work to the feature branch
   ```bash
   git add .
   git commit -m "feat: your feature description"
   ```

3. **Push to remote**:
   ```bash
   git push origin feature/your-feature-name
   ```

4. **Create a Pull Request**: Open a PR on GitHub to merge into `main`

5. **Wait for CI**: All CI checks must pass before merging

6. **Merge**: Once approved and CI passes, merge the PR

**Why branch protection?**
- Ensures all changes go through CI validation
- Maintains code quality and prevents breaking changes
- Creates a reviewable history of changes
- Protects against accidental pushes to production branch

### Working with Pull Requests

```bash
# Create and switch to a new branch
git checkout -b fix/issue-description

# Make changes, commit
git commit -m "fix: description"

# Push and create PR
git push origin fix/issue-description
gh pr create --title "Fix: description" --body "Details about the fix"

# After PR is merged, clean up
git checkout main
git pull
git branch -d fix/issue-description
```

## Release Process

This project uses [tagpr](https://github.com/Songmu/tagpr) for automated release management.

### How tagpr Works

1. **During Development**: Add your changes under the `## [Unreleased]` section in CHANGELOG.md
   ```markdown
   ## [Unreleased]

   ### Added
   - New feature description

   ### Fixed
   - Bug fix description
   ```

2. **Automatic PR Creation**: When you push to `main`, tagpr automatically:
   - Creates/updates a release PR
   - Bumps the version in `Cargo.toml` based on commit messages
   - Updates CHANGELOG.md with the new version and date

3. **Release**: When you merge the tagpr PR:
   - tagpr automatically creates a git tag (e.g., `v0.1.1`)
   - The release workflow triggers (`.github/workflows/release.yml`)
   - Package is published to crates.io
   - GitHub Release is created

### Version Bumping Rules

tagpr determines version bumps from commit messages:
- **Major**: Breaking changes (e.g., `feat!: breaking change` or `BREAKING CHANGE:` in commit body)
- **Minor**: New features (e.g., `feat: add new feature`)
- **Patch**: Bug fixes and other changes (e.g., `fix: correct bug`, `docs: update README`)

### Manual Override

If you need to manually specify the version:
1. Edit the tagpr release PR
2. Change the version in Cargo.toml
3. Update the CHANGELOG.md header
4. Merge the PR

### Configuration

The tagpr configuration is in `.tagpr`:
```ini
[tagpr]
	vPrefix = true
	releaseBranch = main
	versionFile = Cargo.toml
	changelog = true
	command = cargo build --release
```

## Project Status (as of 2026-01-27)

### ✅ Completed
- **Core library**: All essential trie operations implemented
- **Binary compatibility**: Byte-for-byte identical output to C++ version
- **Test coverage**: 321 tests passing (ported from C++ + Rust-specific)
- **CLI tools**: All 6 tools implemented and verified
  - `rsmarisa-build` - Dictionary builder
  - `rsmarisa-lookup` - Key lookup
  - `rsmarisa-common-prefix-search` - Prefix search
  - `rsmarisa-predictive-search` - Predictive search
  - `rsmarisa-reverse-lookup` - ID to key conversion
  - `rsmarisa-dump` - Dictionary dumper
- **Integration tests**: CLI tools verified against C++ counterparts
- **Memory safety**: Fixed all use-after-free bugs in reverse_lookup and predictive_search
- **Published to crates.io**: Version 0.2.0 available at https://crates.io/crates/rsmarisa
- **Automated releases**: Using tagpr for release management and GitHub Actions for CI/CD
- **Memory-mapped I/O**: Full implementation using memmap2 (v0.2.0)
  - `Trie::mmap(filename)` - Load from file-backed memory map
  - `Trie::map(data)` - Load from static memory
  - Binary compatible with C++-created dictionaries
  - All vector types support map() operation
  - Proper lifetime management to prevent dangling pointers
- **Library name alignment**: Changed library name from `marisa` to `rsmarisa` (v0.3.0, merged in PR #8)
  - Eliminates confusion between package name and import path
  - Users now write `rsmarisa = "0.3"` in Cargo.toml and `use rsmarisa::` in code
  - Breaking change requiring update of all imports

### 📝 Known Issues
- Numerous compiler warnings (mostly lifetime annotations) - non-critical

### 🎯 Future Work
- Add benchmarking tool (`rsmarisa-benchmark`)
- Clean up compiler warnings
- Performance optimization

## References

- Original repository: https://github.com/s-yata/marisa-trie
- Baseline version: 0.3.1
- Baseline commit: `4ef33cc5a2b6b4f5e147e4564a5236e163d67982`
- Original license: BSD-2-Clause OR LGPL-2.1-or-later

## Memory-Mapped I/O Implementation

### Overview

rust-marisa now supports true memory-mapped file I/O using the `memmap2` crate. This provides efficient loading of large dictionaries without copying data into memory.

### Architecture

**Mapper without lifetime parameter**: The original design used `Mapper<'a>` with a borrowed lifetime, which made it impossible to own file-backed memory maps. The refactored design uses `Mapper` (no lifetime) that can own either:
- File-backed `Mmap` (from memmap2)
- Borrowed `&'static [u8]` memory

**Ownership model**:
```rust
pub struct Mapper {
    mmap: Option<Mmap>,              // File-backed mmap
    borrowed: Option<&'static [u8]>, // Borrowed memory
    position: usize,
}
```

**Critical: Drop order safety**: The `mapper` field in `LoudsTrie` is placed LAST in the struct declaration. Rust drops fields in declaration order (top to bottom), so this ensures all data structures referencing mmap'd memory are dropped before the `Mapper` itself, preventing dangling pointers.

### Public API

```rust
// File-backed memory mapping (recommended for large dictionaries)
let mut trie = Trie::new();
trie.mmap("dictionary.marisa")?;

// Static memory mapping (for embedded data)
static DATA: &[u8] = include_bytes!("dictionary.marisa");
let mut trie = Trie::new();
trie.map(DATA)?;

// Traditional read (still supported)
let mut trie = Trie::new();
trie.load("dictionary.marisa")?;
```

### Implementation Details

All vector types now support both `read()` and `map()` operations:
- `Vector<T>::map()` - Generic vector mapping
- `BitVector::map()` - Bit vector with rank/select indices
- `FlatVector::map()` - Flat vector for packed integers
- `Tail::map()` - Tail storage for suffixes

The binary format is identical between `read()` and `map()`, ensuring:
- Files created by Rust can be loaded by C++ (via both methods)
- Files created by C++ can be loaded by Rust (via both methods)
- `load()` and `mmap()` produce identical behavior

### Safety Considerations

**Memory mapping safety**: The `unsafe` block in `Mapper::open_file()` is required because:
- File could be modified externally while mapped
- File could be truncated while mapped

**Mitigations**:
- Files are opened read-only (PROT_READ)
- Documentation warns users not to modify files while mapped
- Matches C++ behavior exactly

### Testing

Added comprehensive tests:
- `test_trie_mmap` - Basic mmap functionality
- `test_trie_mmap_vs_load_equivalence` - Verify identical behavior with load()
- `test_trie_mmap_file_not_found` - Error handling
- Integration test with C++-created dictionaries

### Performance Characteristics

**Large dictionaries (>100MB)**:
- Load time: O(1) instant startup vs O(n) read/parse
- Memory: OS page cache (shared) vs heap allocation
- First access: Slower (page fault) then fast

**Small dictionaries (<1MB)**:
- Minimal benefit, setup overhead may dominate
- Use `load()` for simplicity

## Key Lessons Learned

### 1. Use-After-Free Prevention
The most critical bug encountered was use-after-free when storing raw pointers to temporary data. Always ensure pointers reference long-lived data owned by the parent structure.

### 2. Binary Compatibility Debugging
When output doesn't match C++:
1. Add extensive debug logging to both implementations
2. Compare step-by-step to find divergence point
3. Verify data structure layouts match exactly
4. Check sort order and comparison functions
5. Use `cmp` for binary file comparison

### 3. Integration Testing
Testing against the original C++ implementation is invaluable:
- Catches subtle behavioral differences
- Ensures true compatibility
- Provides confidence in correctness
- Documents expected behavior

### 4. Library Name Consistency
**Problem**: Originally, the package name was `rsmarisa` but the library name was `marisa`. This created confusion where users would write `rsmarisa = "0.2"` in their `Cargo.toml` but then use `use marisa::` in their code.

**Solution**: Align the library name with the package name by changing `[lib] name = "marisa"` to `name = "rsmarisa"` in `Cargo.toml`.

**Why this matters**:
- **User experience**: The import path should match what users specify in their dependencies
- **Discoverability**: When searching for "rsmarisa", the import path should be obvious
- **Convention**: Most Rust crates use the same name for package and library
- **Documentation clarity**: Examples and docs become more consistent

**Impact**: This is a breaking change requiring all users to update their imports from `use marisa::` to `use rsmarisa::`. However, it significantly improves the long-term developer experience.

**Files affected**: All source files, tests, examples, and documentation that import the library.
