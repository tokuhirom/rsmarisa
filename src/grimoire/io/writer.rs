//! Writer for serializing trie data.
//!
//! Ported from:
//! - lib/marisa/grimoire/io/writer.h
//! - lib/marisa/grimoire/io/writer.cc

/// Writer for writing trie data to files or memory.
pub struct Writer {
    // TODO: implement
}

impl Writer {
    /// Creates a new writer.
    pub fn new() -> Self {
        Writer {}
    }
}

impl Default for Writer {
    fn default() -> Self {
        Self::new()
    }
}
