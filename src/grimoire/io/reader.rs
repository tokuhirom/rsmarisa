//! Reader for deserializing trie data.
//!
//! Ported from:
//! - lib/marisa/grimoire/io/reader.h
//! - lib/marisa/grimoire/io/reader.cc

/// Reader for reading trie data from files or memory.
pub struct Reader {
    // TODO: implement
}

impl Reader {
    /// Creates a new reader.
    pub fn new() -> Self {
        Reader {}
    }
}

impl Default for Reader {
    fn default() -> Self {
        Self::new()
    }
}
