# Contributing to rsmarisa

Thank you for your interest in contributing to rsmarisa!

## Project Guidelines

Please read [CLAUDE.md](CLAUDE.md) for detailed porting guidelines and project structure.

## Development Workflow

1. **Check porting status**: See [PORTING_STATUS.md](PORTING_STATUS.md) to find what needs to be ported
2. **Read the original code**: Always read the corresponding C++ files first
3. **Create Rust module**: Port the functionality following Rust idioms
4. **Add source attribution**: Include `//! Ported from:` comments
5. **Port tests**: Ensure test coverage matches or exceeds the original
6. **Update status**: Mark your progress in PORTING_STATUS.md
7. **Run tests**: `cargo test` should pass
8. **Run formatting**: `cargo fmt`
9. **Check clippy**: `cargo clippy -- -D warnings`
10. **Commit**: Write clear commit messages in English

## Commit Message Format

Follow conventional commits:

- `feat: add bit vector implementation`
- `fix: correct rank calculation in bit vector`
- `docs: add module documentation for grimoire::io`
- `test: port trie-test.cc test cases`
- `refactor: simplify louds-trie node structure`

## Porting Principles

### DO

- ✅ Mirror the original C++ structure
- ✅ Preserve data structures and algorithms
- ✅ Document source file mapping clearly
- ✅ Port all test cases
- ✅ Use Rust idioms (Option, Result, iterators)
- ✅ Add proper error handling
- ✅ Write documentation in English
- ✅ Keep parallel structure for easy diffing

### DON'T

- ❌ Prematurely optimize or refactor
- ❌ Skip test porting
- ❌ Omit source attribution
- ❌ Break from original structure without good reason
- ❌ Use unsafe without documentation and justification

## Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Use `snake_case` for functions and variables
- Use `CamelCase` for types
- Add doc comments for public items
- Keep line length reasonable (100-120 chars)

## Testing

- Port all original tests
- Add new tests for Rust-specific functionality
- Ensure tests are deterministic
- Use descriptive test names

## Questions?

Feel free to open an issue for discussion before starting major work.

## License

By contributing, you agree that your contributions will be licensed under the BSD-2-Clause license.
