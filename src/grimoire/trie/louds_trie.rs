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

    // TODO: Implement I/O methods (map, read, write)
    // These require BitVector, FlatVector, and Tail to have proper I/O support

    // Helper methods

    /// Gets cache ID from node ID and label.
    #[inline]
    fn get_cache_id_with_label(&self, node_id: usize, label: u8) -> usize {
        (node_id ^ (node_id << 5) ^ (label as usize)) & self.cache_mask
    }

    /// Gets cache ID from node ID only.
    #[inline]
    fn get_cache_id(&self, node_id: usize) -> usize {
        node_id & self.cache_mask
    }

    /// Gets link value from a node.
    #[inline]
    fn get_link_simple(&self, node_id: usize) -> usize {
        let base = self.bases[node_id] as usize;
        let extra_idx = self.link_flags.rank1(node_id);
        let extra = self.extras.get(extra_idx) as usize;
        base | (extra * 256)
    }

    /// Gets link value from a node with specific link ID.
    #[inline]
    fn get_link_with_id(&self, node_id: usize, link_id: usize) -> usize {
        let base = self.bases[node_id] as usize;
        let extra = self.extras.get(link_id) as usize;
        base | (extra * 256)
    }

    /// Updates link ID for iteration.
    #[inline]
    fn update_link_id(&self, link_id: usize, node_id: usize) -> usize {
        use crate::base::INVALID_LINK_ID;
        if link_id == INVALID_LINK_ID as usize {
            self.link_flags.rank1(node_id)
        } else {
            link_id + 1
        }
    }

    /// Restores a key from a link.
    ///
    /// Delegates to either next trie or tail storage.
    #[inline]
    fn restore(&self, agent: &mut crate::agent::Agent, link: usize) {
        if let Some(ref next) = self.next_trie {
            next.restore_(agent, link);
        } else {
            self.tail.restore(agent, link);
        }
    }

    /// Matches query against a link.
    ///
    /// Delegates to either next trie or tail storage.
    #[inline]
    fn match_link(&self, agent: &mut crate::agent::Agent, link: usize) -> bool {
        if let Some(ref next) = self.next_trie {
            next.match_(agent, link)
        } else {
            self.tail.match_tail(agent, link)
        }
    }

    /// Matches query prefix and restores the rest from a link.
    ///
    /// Delegates to either next trie or tail storage.
    #[inline]
    fn prefix_match(&self, agent: &mut crate::agent::Agent, link: usize) -> bool {
        if let Some(ref next) = self.next_trie {
            next.prefix_match_(agent, link)
        } else {
            self.tail.prefix_match(agent, link)
        }
    }

    /// Internal restore implementation for recursive calls.
    fn restore_(&self, agent: &mut crate::agent::Agent, node_id: usize) {
        assert!(node_id != 0, "Node ID must not be 0");

        let mut node_id = node_id;

        loop {
            let cache_id = self.get_cache_id(node_id);
            if node_id == self.cache[cache_id].child() {
                use crate::base::INVALID_EXTRA;
                if self.cache[cache_id].extra() != INVALID_EXTRA as usize {
                    self.restore(agent, self.cache[cache_id].link());
                } else {
                    agent
                        .state_mut()
                        .expect("Agent must have state")
                        .key_buf_mut()
                        .push(self.cache[cache_id].label());
                }

                node_id = self.cache[cache_id].parent();
                if node_id == 0 {
                    return;
                }
                continue;
            }

            if self.link_flags.get(node_id) {
                self.restore(agent, self.get_link_simple(node_id));
            } else {
                agent
                    .state_mut()
                    .expect("Agent must have state")
                    .key_buf_mut()
                    .push(self.bases[node_id]);
            }

            if node_id <= self.num_l1_nodes {
                return;
            }
            node_id = self.louds.select1(node_id) - node_id - 1;
        }
    }

    /// Internal match implementation for recursive calls.
    fn match_(&self, agent: &mut crate::agent::Agent, node_id: usize) -> bool {
        let query_len = agent.query().length();
        let mut query_pos = agent
            .state()
            .expect("Agent must have state")
            .query_pos();

        assert!(query_pos < query_len, "Query position out of bounds");
        assert!(node_id != 0, "Node ID must not be 0");

        let query_bytes = agent.query().as_bytes().to_vec();
        let mut node_id = node_id;

        loop {
            let cache_id = self.get_cache_id(node_id);
            if node_id == self.cache[cache_id].child() {
                use crate::base::INVALID_EXTRA;
                if self.cache[cache_id].extra() != INVALID_EXTRA as usize {
                    if !self.match_link(agent, self.cache[cache_id].link()) {
                        return false;
                    }
                } else if self.cache[cache_id].label() == query_bytes[query_pos] {
                    query_pos += 1;
                    agent
                        .state_mut()
                        .expect("Agent must have state")
                        .set_query_pos(query_pos);
                } else {
                    return false;
                }

                node_id = self.cache[cache_id].parent();
                if node_id == 0 {
                    return true;
                }
                if query_pos >= query_len {
                    return false;
                }
                continue;
            }

            if self.link_flags.get(node_id) {
                if let Some(ref next) = self.next_trie {
                    if !self.match_link(agent, self.get_link_simple(node_id)) {
                        return false;
                    }
                } else if !self.tail.match_tail(agent, self.get_link_simple(node_id)) {
                    return false;
                }
            } else if self.bases[node_id] == query_bytes[query_pos] {
                query_pos += 1;
                agent
                    .state_mut()
                    .expect("Agent must have state")
                    .set_query_pos(query_pos);
            } else {
                return false;
            }

            if node_id <= self.num_l1_nodes {
                return true;
            }
            if query_pos >= query_len {
                return false;
            }
            node_id = self.louds.select1(node_id) - node_id - 1;
        }
    }

    /// Internal prefix match implementation for recursive calls.
    fn prefix_match_(&self, agent: &mut crate::agent::Agent, node_id: usize) -> bool {
        let query_len = agent.query().length();
        let mut query_pos = agent
            .state()
            .expect("Agent must have state")
            .query_pos();

        assert!(query_pos < query_len, "Query position out of bounds");
        assert!(node_id != 0, "Node ID must not be 0");

        let query_bytes = agent.query().as_bytes().to_vec();
        let mut node_id = node_id;

        loop {
            let cache_id = self.get_cache_id(node_id);
            if node_id == self.cache[cache_id].child() {
                use crate::base::INVALID_EXTRA;
                if self.cache[cache_id].extra() != INVALID_EXTRA as usize {
                    if !self.prefix_match(agent, self.cache[cache_id].link()) {
                        return false;
                    }
                } else if self.cache[cache_id].label() == query_bytes[query_pos] {
                    agent
                        .state_mut()
                        .expect("Agent must have state")
                        .key_buf_mut()
                        .push(self.cache[cache_id].label());
                    query_pos += 1;
                    agent
                        .state_mut()
                        .expect("Agent must have state")
                        .set_query_pos(query_pos);
                } else {
                    return false;
                }

                node_id = self.cache[cache_id].parent();
                if node_id == 0 {
                    return true;
                }
            } else {
                if self.link_flags.get(node_id) {
                    if !self.prefix_match(agent, self.get_link_simple(node_id)) {
                        return false;
                    }
                } else if self.bases[node_id] == query_bytes[query_pos] {
                    agent
                        .state_mut()
                        .expect("Agent must have state")
                        .key_buf_mut()
                        .push(self.bases[node_id]);
                    query_pos += 1;
                    agent
                        .state_mut()
                        .expect("Agent must have state")
                        .set_query_pos(query_pos);
                } else {
                    return false;
                }

                if node_id <= self.num_l1_nodes {
                    return true;
                }
                node_id = self.louds.select1(node_id) - node_id - 1;
            }

            if query_pos >= query_len {
                self.restore_(agent, node_id);
                return true;
            }
        }
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
