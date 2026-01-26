# Contributing to rsmarisa

Thank you for your interest in contributing to rsmarisa!

## Project Guidelines

Please read [CLAUDE.md](CLAUDE.md) for detailed porting guidelines and project structure.

## Development Workflow

**Note**: The `main` branch is protected. All changes must go through pull requests.

1. **Create a feature branch**: `git checkout -b feat/your-feature`
2. **Check porting status**: See [PORTING_STATUS.md](PORTING_STATUS.md) to find what needs to be ported
3. **Read the original code**: Always read the corresponding C++ files first
4. **Create Rust module**: Port the functionality following Rust idioms
5. **Add source attribution**: Include `//! Ported from:` comments
6. **Port tests**: Ensure test coverage matches or exceeds the original
7. **Update status**: Mark your progress in PORTING_STATUS.md
8. **Run tests**: `cargo test` should pass
9. **Run formatting**: `cargo fmt`
10. **Check clippy**: `cargo clippy -- -D warnings`
11. **Update CHANGELOG.md**: Add your changes under the `[Unreleased]` section
12. **Commit**: Write clear commit messages in English
13. **Push and create PR**: `git push origin feat/your-feature` then create a pull request
14. **Wait for CI**: All checks must pass before merging

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

## Release Process

This project uses [tagpr](https://github.com/Songmu/tagpr) for automated release management.

### For Contributors

When making changes, add them to the `[Unreleased]` section in CHANGELOG.md:

```markdown
## [Unreleased]

### Added
- New feature description

### Fixed
- Bug fix description
```

Use conventional commit messages:
- `feat:` for new features (minor version bump)
- `fix:` for bug fixes (patch version bump)
- `feat!:` or `BREAKING CHANGE:` for breaking changes (major version bump)

### For Maintainers

1. tagpr automatically creates/updates a release PR when changes are pushed to main
2. Review the PR - it will contain version bump and CHANGELOG updates
3. Merge the PR to trigger the release:
   - Creates a git tag
   - Publishes to crates.io
   - Creates a GitHub Release

## Questions?

Feel free to open an issue for discussion before starting major work.

## License

By contributing, you agree that your contributions will be licensed under the BSD-2-Clause license.
