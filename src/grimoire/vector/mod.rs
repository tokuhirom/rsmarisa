//! Vector data structures for compact storage.
//!
//! Ported from: lib/marisa/grimoire/vector/
//!
//! This module provides specialized vector implementations:
//! - Bit vector: compact binary data with rank/select operations
//! - Flat vector: space-efficient integer vector
//! - Rank index: rank operation acceleration

pub mod bit_vector;
pub mod flat_vector;
pub mod rank_index;
#[allow(clippy::module_inception)]
pub mod vector;
