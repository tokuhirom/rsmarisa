# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.2.1](https://github.com/tokuhirom/rsmarisa/compare/v0.2.0...v0.2.1) - 2026-01-27
- docs: update README with memory-mapped I/O feature by @tokuhirom in https://github.com/tokuhirom/rsmarisa/pull/6
- fix: align library name with package name (marisa -> rsmarisa) by @tokuhirom in https://github.com/tokuhirom/rsmarisa/pull/8
- docs: document library name change and migration guide by @tokuhirom in https://github.com/tokuhirom/rsmarisa/pull/9

## [v0.2.0](https://github.com/tokuhirom/rsmarisa/compare/v0.1.0...v0.2.0) - 2026-01-27
- docs: add branch protection and PR workflow information by @tokuhirom in https://github.com/tokuhirom/rsmarisa/pull/1
- feat: add tagpr for automated release management by @tokuhirom in https://github.com/tokuhirom/rsmarisa/pull/2
- feat: add memory-mapped I/O support using memmap2 by @tokuhirom in https://github.com/tokuhirom/rsmarisa/pull/3
- Update tagpr.yml by @tokuhirom in https://github.com/tokuhirom/rsmarisa/pull/4

## [Unreleased]

### Changed

- **BREAKING**: Library name changed from `marisa` to `rsmarisa` to align with package name
  - Users must update imports from `use marisa::` to `use rsmarisa::`
  - Eliminates confusion where package name (`rsmarisa`) didn't match import path (`marisa`)
  - Improves discoverability and follows Rust naming conventions
  - All source files, tests, examples, and documentation updated

## [0.1.0] - 2026-01-26

### Added

- Initial release of rsmarisa, a pure Rust port of marisa-trie
- LOUDS (Level-Order Unary Degree Sequence) trie implementation
- Core operations:
  - Lookup: exact string matching
  - Reverse lookup: restore key from ID
  - Common prefix search: find all prefixes of a query string
  - Predictive search: find all keys with a given prefix
- Binary I/O with full C++ marisa-trie compatibility
  - Save and load tries to/from files
  - Byte-for-byte identical output to C++ marisa-trie 0.3.1
- CLI tools with `rsmarisa-` prefix:
  - `rsmarisa-build`: build a dictionary from text input
  - `rsmarisa-lookup`: look up keys in a dictionary
  - `rsmarisa-common-prefix-search`: find prefix matches
  - `rsmarisa-predictive-search`: find keys with given prefix
  - `rsmarisa-reverse-lookup`: restore keys from IDs
  - `rsmarisa-dump`: dump dictionary contents
- Comprehensive test suite with 314 tests
- Examples demonstrating basic usage and file I/O

### Fixed

- Use-after-free bugs in `reverse_lookup()` and `predictive_search()`
  - Fixed memory safety issues where keys pointed to freed temporary buffers
  - Now properly uses agent's state buffer for key storage
- Lookup failures for tries with 7+ keys
  - Fixed ReverseKey substring extraction in tail building
  - Corrected reverse index calculation in `build_current_trie_reverse()`
- Tail entry sort order to match C++ behavior (ascending order)
- Binary compatibility issues ensuring byte-identical output
- Query position synchronization in multi-trie `match_()` operations

### Technical Details

- Rust edition: 2021
- Minimum supported Rust version (MSRV): 1.70
- Based on marisa-trie 0.3.1 (commit: 4ef33cc5)
- License: BSD-2-Clause (same as original)

[0.1.0]: https://github.com/tokuhirom/rsmarisa/releases/tag/v0.1.0
