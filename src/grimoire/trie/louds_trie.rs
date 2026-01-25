//! LOUDS (Level-Order Unary Degree Sequence) trie implementation.
//!
//! Ported from:
//! - lib/marisa/grimoire/trie/louds-trie.h
//! - lib/marisa/grimoire/trie/louds-trie.cc
//!
//! This is the core trie data structure using LOUDS encoding for
//! space-efficient storage while maintaining fast search operations.

use crate::grimoire::io::{Mapper, Reader, Writer};
use crate::grimoire::trie::cache::Cache;
use crate::grimoire::trie::config::Config;
use crate::grimoire::trie::tail::Tail;
use crate::grimoire::vector::bit_vector::BitVector;
use crate::grimoire::vector::flat_vector::FlatVector;
use crate::grimoire::vector::vector::Vector;
use crate::base::{CacheLevel, TailMode, NodeOrder};

/// LOUDS-based trie structure.
///
/// LOUDS (Level-Order Unary Degree Sequence) is a succinct data structure
/// for representing trees. This implementation uses it for trie storage.
pub struct LoudsTrie {
    /// LOUDS bit vector for trie structure.
    louds: BitVector,
    /// Terminal flags (1 = this node represents end of a key).
    terminal_flags: BitVector,
    /// Link flags (1 = this node links to tail storage).
    link_flags: BitVector,
    /// Base values for each node (labels or offsets).
    bases: Vector<u8>,
    /// Extra data for nodes (packed integers).
    extras: FlatVector,
    /// Tail storage for suffixes.
    tail: Tail,
    /// Next trie in the multi-trie structure.
    next_trie: Option<Box<LoudsTrie>>,
    /// Cache for accelerating searches.
    cache: Vector<Cache>,
    /// Mask for cache access.
    cache_mask: usize,
    /// Number of level-1 nodes.
    num_l1_nodes: usize,
    /// Configuration.
    config: Config,
    /// Mapper for memory-mapped access.
    mapper: Mapper<'static>,
}

impl Default for LoudsTrie {
    fn default() -> Self {
        Self::new()
    }
}

impl LoudsTrie {
    /// Creates a new empty LOUDS trie.
    pub fn new() -> Self {
        LoudsTrie {
            louds: BitVector::new(),
            terminal_flags: BitVector::new(),
            link_flags: BitVector::new(),
            bases: Vector::new(),
            extras: FlatVector::new(),
            tail: Tail::new(),
            next_trie: None,
            cache: Vector::new(),
            cache_mask: 0,
            num_l1_nodes: 0,
            config: Config::new(),
            mapper: Mapper::new(),
        }
    }

    /// Returns the number of tries in the multi-trie structure.
    pub fn num_tries(&self) -> usize {
        self.config.num_tries()
    }

    /// Returns the number of keys stored in the trie.
    pub fn num_keys(&self) -> usize {
        self.size()
    }

    /// Returns the number of nodes in the trie.
    pub fn num_nodes(&self) -> usize {
        (self.louds.size() / 2).saturating_sub(1)
    }

    /// Returns the cache level configuration.
    pub fn cache_level(&self) -> CacheLevel {
        self.config.cache_level()
    }

    /// Returns the tail mode configuration.
    pub fn tail_mode(&self) -> TailMode {
        self.config.tail_mode()
    }

    /// Returns the node order configuration.
    pub fn node_order(&self) -> NodeOrder {
        self.config.node_order()
    }

    /// Returns true if the trie is empty.
    pub fn empty(&self) -> bool {
        self.size() == 0
    }

    /// Returns the number of keys in the trie.
    pub fn size(&self) -> usize {
        self.terminal_flags.num_1s()
    }

    /// Returns the total size in bytes.
    pub fn total_size(&self) -> usize {
        self.louds.total_size()
            + self.terminal_flags.total_size()
            + self.link_flags.total_size()
            + self.bases.total_size()
            + self.extras.total_size()
            + self.tail.total_size()
            + self.next_trie.as_ref().map_or(0, |t| t.total_size())
            + self.cache.total_size()
            + std::mem::size_of::<Self>()
    }

    /// Returns the I/O size in bytes.
    pub fn io_size(&self) -> usize {
        self.louds.io_size()
            + self.terminal_flags.io_size()
            + self.link_flags.io_size()
            + self.bases.io_size()
            + self.extras.io_size()
            + self.tail.io_size()
            + self.next_trie.as_ref().map_or(0, |t| t.io_size())
    }

    /// Clears the trie to empty state.
    pub fn clear(&mut self) {
        *self = LoudsTrie::new();
    }

    /// Swaps with another trie.
    pub fn swap(&mut self, other: &mut LoudsTrie) {
        std::mem::swap(self, other);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_louds_trie_new() {
        let trie = LoudsTrie::new();
        assert!(trie.empty());
        assert_eq!(trie.size(), 0);
        assert_eq!(trie.num_nodes(), 0);
    }

    #[test]
    fn test_louds_trie_default() {
        let trie = LoudsTrie::default();
        assert!(trie.empty());
    }

    #[test]
    fn test_louds_trie_clear() {
        let mut trie = LoudsTrie::new();
        // Would need build() to populate
        trie.clear();
        assert!(trie.empty());
        assert_eq!(trie.size(), 0);
    }

    #[test]
    fn test_louds_trie_swap() {
        let mut t1 = LoudsTrie::new();
        let mut t2 = LoudsTrie::new();

        t1.swap(&mut t2);

        assert!(t1.empty());
        assert!(t2.empty());
    }

    #[test]
    fn test_louds_trie_accessors() {
        let trie = LoudsTrie::new();

        // Check that accessors work
        assert_eq!(trie.num_keys(), 0);
        assert_eq!(trie.cache_level(), CacheLevel::Normal);
        assert_eq!(trie.tail_mode(), TailMode::TextTail);
        assert_eq!(trie.node_order(), NodeOrder::Weight);
    }
}
