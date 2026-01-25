//! Bit vector with rank/select operations.
//!
//! Ported from:
//! - lib/marisa/grimoire/vector/bit-vector.h
//! - lib/marisa/grimoire/vector/bit-vector.cc

/// Bit vector supporting rank and select operations.
pub struct BitVector {
    // TODO: implement
}

impl BitVector {
    /// Creates a new bit vector.
    pub fn new() -> Self {
        BitVector {}
    }
}

impl Default for BitVector {
    fn default() -> Self {
        Self::new()
    }
}
