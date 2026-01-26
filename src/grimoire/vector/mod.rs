//! Vector data structures for compact storage.
//!
//! Ported from: lib/marisa/grimoire/vector/
//!
//! This module provides specialized vector implementations:
//! - Bit vector: compact binary data with rank/select operations
//! - Flat vector: space-efficient integer vector
//! - Pop count: population count utilities
//! - Rank index: rank operation acceleration
//! - Select tables: lookup tables for select operations

pub mod bit_vector;
pub mod flat_vector;
pub mod pop_count;
pub mod rank_index;
pub mod select_bit;
pub mod select_tables;
#[allow(clippy::module_inception)]
pub mod vector;
