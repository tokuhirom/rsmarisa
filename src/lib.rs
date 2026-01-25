//! # marisa
//!
//! Rust port of marisa-trie: a static and space-efficient trie data structure.
//!
//! Ported from: include/marisa.h
//!
//! ## About MARISA
//!
//! MARISA (Matching Algorithm with Recursively Implemented StorAge) is a static
//! and space-efficient trie data structure. This Rust implementation aims to
//! maintain compatibility with the original C++ implementation while leveraging
//! Rust's safety features.
//!
//! ## Features
//!
//! A MARISA-based dictionary supports:
//! - **Lookup**: Check whether a given string exists in the dictionary
//! - **Reverse lookup**: Restore a key from its ID
//! - **Common prefix search**: Find keys from prefixes of a given string
//! - **Predictive search**: Find keys starting with a given string
//!
//! ## Original Project
//!
//! This is a Rust port of [marisa-trie](https://github.com/s-yata/marisa-trie)
//! originally written by Susumu Yata.
//!
//! - Original version: 0.3.1
//! - Baseline commit: 4ef33cc5a2b6b4f5e147e4564a5236e163d67982
//! - Original license: BSD-2-Clause OR LGPL-2.1-or-later

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod base;
pub mod grimoire;
pub mod key;
pub mod keyset;
pub mod query;

// Re-export main types at the crate root
// These correspond to the public API in include/marisa/*.h
