//! Trie header for serialization.
//!
//! Ported from: lib/marisa/grimoire/trie/header.h

/// Header containing trie metadata.
pub struct Header {
    // TODO: implement
}

impl Header {
    /// Creates a new header.
    pub fn new() -> Self {
        Header {}
    }
}

impl Default for Header {
    fn default() -> Self {
        Self::new()
    }
}
