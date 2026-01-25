//! Tail storage for trie suffixes.
//!
//! Ported from:
//! - lib/marisa/grimoire/trie/tail.h
//! - lib/marisa/grimoire/trie/tail.cc

/// Tail structure for storing common suffixes.
pub struct Tail {
    // TODO: implement
}

impl Tail {
    /// Creates a new tail.
    pub fn new() -> Self {
        Tail {}
    }
}

impl Default for Tail {
    fn default() -> Self {
        Self::new()
    }
}
