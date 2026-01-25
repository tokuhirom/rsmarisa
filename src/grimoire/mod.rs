//! Internal implementation modules.
//!
//! Ported from: lib/marisa/grimoire/
//!
//! The grimoire modules contain the internal implementation details of the
//! MARISA trie data structure, including:
//! - I/O operations (reader, writer, mapper)
//! - Trie structures (LOUDS trie, tail, etc.)
//! - Vector implementations (bit vector, flat vector)
//! - Algorithms (sorting, etc.)

pub mod algorithm;
pub mod io;
pub mod trie;
pub mod vector;
