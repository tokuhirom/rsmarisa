//! Memory-mapped file access.
//!
//! Ported from:
//! - lib/marisa/grimoire/io/mapper.h
//! - lib/marisa/grimoire/io/mapper.cc

/// Mapper for memory-mapped file access.
pub struct Mapper {
    // TODO: implement
}

impl Mapper {
    /// Creates a new mapper.
    pub fn new() -> Self {
        Mapper {}
    }
}

impl Default for Mapper {
    fn default() -> Self {
        Self::new()
    }
}
