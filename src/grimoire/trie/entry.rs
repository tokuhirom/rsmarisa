//! Entry type for trie construction.
//!
//! Ported from: lib/marisa/grimoire/trie/entry.h

/// Entry representing a key during trie construction.
pub struct Entry {
    // TODO: implement
}

impl Entry {
    /// Creates a new entry.
    pub fn new() -> Self {
        Entry {}
    }
}

impl Default for Entry {
    fn default() -> Self {
        Self::new()
    }
}
