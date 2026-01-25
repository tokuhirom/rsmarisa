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

    /// Looks up a key in the trie.
    ///
    /// Returns true if the query string exists as a complete key in the trie.
    /// Sets the agent's key to the matched string and its ID.
    ///
    /// # Arguments
    ///
    /// * `agent` - Agent with initialized state and query
    ///
    /// # Panics
    ///
    /// Panics if agent doesn't have state initialized.
    pub fn lookup(&self, agent: &mut crate::agent::Agent) -> bool {
        assert!(agent.has_state(), "Agent must have state initialized");

        // Initialize for lookup
        {
            let state = agent.state_mut().expect("Agent must have state");
            state.lookup_init();
        }

        // Traverse trie following query
        let query_len = agent.query().length();
        while agent.state().expect("Agent must have state").query_pos() < query_len {
            if !self.find_child(agent) {
                return false;
            }
        }

        // Check if this node is a terminal (end of a key)
        let node_id = agent.state().expect("Agent must have state").node_id();
        if !self.terminal_flags.get(node_id) {
            return false;
        }

        // Set result key
        let query_bytes = agent.query().as_bytes().to_vec();
        agent.set_key_bytes(&query_bytes);
        let key_id = self.terminal_flags.rank1(node_id);
        agent.set_key_id(key_id);

        true
    }

    /// Performs reverse lookup: finds the key corresponding to a key ID.
    ///
    /// Reconstructs the key string from its ID by traversing the trie.
    ///
    /// # Arguments
    ///
    /// * `agent` - Agent with initialized state and query containing key ID
    ///
    /// # Panics
    ///
    /// Panics if agent doesn't have state or if key ID is out of range.
    pub fn reverse_lookup(&self, agent: &mut crate::agent::Agent) {
        assert!(agent.has_state(), "Agent must have state initialized");

        let key_id = agent.query().id();
        assert!(key_id < self.size(), "Key ID out of range");

        // Initialize for reverse lookup
        {
            let state = agent.state_mut().expect("Agent must have state");
            state.reverse_lookup_init();
        }

        // Find the terminal node for this key ID
        let node_id = self.terminal_flags.select1(key_id);
        {
            let state = agent.state_mut().expect("Agent must have state");
            state.set_node_id(node_id);
        }

        // Handle root node case
        if node_id == 0 {
            let key_bytes = agent
                .state()
                .expect("Agent must have state")
                .key_buf()
                .to_vec();
            agent.set_key_bytes(&key_bytes);
            agent.set_key_id(key_id);
            return;
        }

        // Traverse upward to root, building key in reverse
        let mut current_node = node_id;
        loop {
            if self.link_flags.get(current_node) {
                // Save position before restore
                let prev_key_pos = agent
                    .state()
                    .expect("Agent must have state")
                    .key_buf()
                    .len();

                self.restore(agent, self.get_link_simple(current_node));

                // Reverse the newly added portion
                let state = agent.state_mut().expect("Agent must have state");
                let key_buf = state.key_buf_mut();
                key_buf[prev_key_pos..].reverse();
            } else {
                agent
                    .state_mut()
                    .expect("Agent must have state")
                    .key_buf_mut()
                    .push(self.bases[current_node]);
            }

            if current_node <= self.num_l1_nodes {
                // Reverse entire key buffer
                let state = agent.state_mut().expect("Agent must have state");
                state.key_buf_mut().reverse();

                let key_bytes = state.key_buf().to_vec();
                drop(state);

                agent.set_key_bytes(&key_bytes);
                agent.set_key_id(key_id);
                return;
            }

            current_node = self.louds.select1(current_node) - current_node - 1;
            agent
                .state_mut()
                .expect("Agent must have state")
                .set_node_id(current_node);
        }
    }

    /// Finds a child node matching the current query character.
    ///
    /// Internal helper for lookup operation.
    fn find_child(&self, agent: &mut crate::agent::Agent) -> bool {
        let state = agent.state().expect("Agent must have state");
        let query_pos = state.query_pos();
        let query_len = agent.query().length();

        assert!(query_pos < query_len, "Query position out of bounds");

        let node_id = state.node_id();
        let query_bytes = agent.query().as_bytes().to_vec();
        let query_char = query_bytes[query_pos];

        // Check cache first
        let cache_id = self.get_cache_id_with_label(node_id, query_char);
        if node_id == self.cache[cache_id].parent() {
            use crate::base::INVALID_EXTRA;
            if self.cache[cache_id].extra() != INVALID_EXTRA as usize {
                if !self.match_link(agent, self.cache[cache_id].link()) {
                    return false;
                }
            } else {
                let new_pos = query_pos + 1;
                agent
                    .state_mut()
                    .expect("Agent must have state")
                    .set_query_pos(new_pos);
            }
            agent
                .state_mut()
                .expect("Agent must have state")
                .set_node_id(self.cache[cache_id].child());
            return true;
        }

        // Search children
        let mut louds_pos = self.louds.select0(node_id) + 1;
        if !self.louds.get(louds_pos) {
            return false;
        }

        let mut current_node = louds_pos - node_id - 1;
        agent
            .state_mut()
            .expect("Agent must have state")
            .set_node_id(current_node);

        let mut link_id = crate::base::INVALID_LINK_ID as usize;

        loop {
            if self.link_flags.get(current_node) {
                link_id = self.update_link_id(link_id, current_node);
                let prev_query_pos = agent.state().expect("Agent must have state").query_pos();

                if self.match_link(agent, self.get_link_with_id(current_node, link_id)) {
                    return true;
                }

                if agent.state().expect("Agent must have state").query_pos() != prev_query_pos {
                    return false;
                }
            } else if self.bases[current_node] == query_char {
                let new_pos = query_pos + 1;
                agent
                    .state_mut()
                    .expect("Agent must have state")
                    .set_query_pos(new_pos);
                return true;
            }

            current_node += 1;
            louds_pos += 1;
            agent
                .state_mut()
                .expect("Agent must have state")
                .set_node_id(current_node);

            if !self.louds.get(louds_pos) {
                break;
            }
        }

        false
    }

    /// Performs common prefix search.
    ///
    /// Finds all keys that are prefixes of the query string.
    /// Call repeatedly to get all matches.
    ///
    /// # Arguments
    ///
    /// * `agent` - Agent with initialized state and query
    ///
    /// # Panics
    ///
    /// Panics if agent doesn't have state initialized.
    pub fn common_prefix_search(&self, agent: &mut crate::agent::Agent) -> bool {
        use crate::grimoire::trie::state::StatusCode;

        assert!(agent.has_state(), "Agent must have state initialized");

        // Check if search is complete
        {
            let state = agent.state().expect("Agent must have state");
            if state.status_code() == StatusCode::EndOfCommonPrefixSearch {
                return false;
            }
        }

        // Initialize on first call
        {
            let state = agent.state().expect("Agent must have state");
            if state.status_code() != StatusCode::ReadyToCommonPrefixSearch {
                drop(state);
                let state = agent.state_mut().expect("Agent must have state");
                state.common_prefix_search_init();

                // Check if root is terminal
                let node_id = state.node_id();
                if self.terminal_flags.get(node_id) {
                    let query_pos = state.query_pos();
                    drop(state);

                    let query_bytes = agent.query().as_bytes().to_vec();
                    agent.set_key_bytes(&query_bytes[..query_pos]);
                    let key_id = self.terminal_flags.rank1(node_id);
                    agent.set_key_id(key_id);
                    return true;
                }
            }
        }

        // Search for prefix matches
        let query_len = agent.query().length();
        while agent.state().expect("Agent must have state").query_pos() < query_len {
            if !self.find_child(agent) {
                agent
                    .state_mut()
                    .expect("Agent must have state")
                    .set_status_code(StatusCode::EndOfCommonPrefixSearch);
                return false;
            }

            let node_id = agent.state().expect("Agent must have state").node_id();
            if self.terminal_flags.get(node_id) {
                let query_pos = agent.state().expect("Agent must have state").query_pos();
                let query_bytes = agent.query().as_bytes().to_vec();
                agent.set_key_bytes(&query_bytes[..query_pos]);
                let key_id = self.terminal_flags.rank1(node_id);
                agent.set_key_id(key_id);
                return true;
            }
        }

        agent
            .state_mut()
            .expect("Agent must have state")
            .set_status_code(StatusCode::EndOfCommonPrefixSearch);
        false
    }

    /// Performs predictive search.
    ///
    /// Finds all keys that start with the query string.
    /// Call repeatedly to get all matches.
    ///
    /// # Arguments
    ///
    /// * `agent` - Agent with initialized state and query
    ///
    /// # Panics
    ///
    /// Panics if agent doesn't have state initialized.
    pub fn predictive_search(&self, agent: &mut crate::agent::Agent) -> bool {
        use crate::grimoire::trie::history::History;
        use crate::grimoire::trie::state::StatusCode;

        assert!(agent.has_state(), "Agent must have state initialized");

        // Check if search is complete
        {
            let state = agent.state().expect("Agent must have state");
            if state.status_code() == StatusCode::EndOfPredictiveSearch {
                return false;
            }
        }

        // Initialize on first call
        {
            let state = agent.state().expect("Agent must have state");
            if state.status_code() != StatusCode::ReadyToPredictiveSearch {
                drop(state);
                let query_len = agent.query().length();
                let state = agent.state_mut().expect("Agent must have state");
                state.predictive_search_init();
                drop(state);
                while agent.state().expect("Agent must have state").query_pos() < query_len {
                    if !self.predictive_find_child(agent) {
                        agent
                            .state_mut()
                            .expect("Agent must have state")
                            .set_status_code(StatusCode::EndOfPredictiveSearch);
                        return false;
                    }
                }
                let state = agent.state_mut().expect("Agent must have state");

                // Push initial history
                let mut history = History::new();
                history.set_node_id(state.node_id());
                history.set_key_pos(state.key_buf().len());
                state.push_history(history);
                state.set_history_pos(1);

                // Check if current node is terminal
                let node_id = state.node_id();
                if self.terminal_flags.get(node_id) {
                    let key_bytes = state.key_buf().to_vec();
                    drop(state);

                    agent.set_key_bytes(&key_bytes);
                    let key_id = self.terminal_flags.rank1(node_id);
                    agent.set_key_id(key_id);
                    return true;
                }
            }
        }

        // Enumerate all keys under current node
        loop {
            let (history_pos, history_size) = {
                let state = agent.state().expect("Agent must have state");
                (state.history_pos(), state.history_size())
            };

            if history_pos == history_size {
                // Need to create next child
                let current_history = {
                    let state = agent.state().expect("Agent must have state");
                    state.history_back().clone()
                };

                let mut next = History::new();
                next.set_louds_pos(self.louds.select0(current_history.node_id()) + 1);
                next.set_node_id(next.louds_pos() - current_history.node_id() - 1);

                agent
                    .state_mut()
                    .expect("Agent must have state")
                    .push_history(next);
            }

            // Get next history entry
            let link_flag = {
                let state = agent.state_mut().expect("Agent must have state");
                let next = state.history_at_mut(history_pos);
                let louds_pos = next.louds_pos();
                let link_flag = self.louds.get(louds_pos);
                next.set_louds_pos(louds_pos + 1);
                link_flag
            };

            if link_flag {
                // This is a child node
                let state = agent.state_mut().expect("Agent must have state");
                state.set_history_pos(history_pos + 1);

                let next_node_id = state.history_at(history_pos).node_id();
                let next_link_id = state.history_at(history_pos).link_id();

                if self.link_flags.get(next_node_id) {
                    let new_link_id = self.update_link_id(next_link_id, next_node_id);
                    state.history_at_mut(history_pos).set_link_id(new_link_id);
                    drop(state);

                    self.restore(agent, self.get_link_with_id(next_node_id, new_link_id));

                    let state = agent.state_mut().expect("Agent must have state");
                    let key_len = state.key_buf().len();
                    state
                        .history_at_mut(history_pos)
                        .set_key_pos(key_len);
                } else {
                    state.key_buf_mut().push(self.bases[next_node_id]);
                    let key_len = state.key_buf().len();
                    state
                        .history_at_mut(history_pos)
                        .set_key_pos(key_len);
                }

                // Check if terminal
                if self.terminal_flags.get(next_node_id) {
                    let state = agent.state_mut().expect("Agent must have state");
                    let next_key_id = state.history_at(history_pos).key_id();

                    use crate::base::INVALID_KEY_ID;
                    let key_id = if next_key_id == INVALID_KEY_ID as usize {
                        let id = self.terminal_flags.rank1(next_node_id);
                        state.history_at_mut(history_pos).set_key_id(id);
                        id
                    } else {
                        let id = next_key_id + 1;
                        state.history_at_mut(history_pos).set_key_id(id);
                        id
                    };

                    let key_bytes = state.key_buf().to_vec();
                    drop(state);

                    agent.set_key_bytes(&key_bytes);
                    agent.set_key_id(key_id);
                    return true;
                }
            } else if history_pos != 1 {
                // Backtrack
                let state = agent.state_mut().expect("Agent must have state");
                let current = state.history_at_mut(history_pos - 1);
                current.set_node_id(current.node_id() + 1);

                let prev_key_pos = state.history_at(history_pos - 2).key_pos();
                state.key_buf_mut().truncate(prev_key_pos);
                state.set_history_pos(history_pos - 1);
            } else {
                // No more results
                agent
                    .state_mut()
                    .expect("Agent must have state")
                    .set_status_code(StatusCode::EndOfPredictiveSearch);
                return false;
            }
        }
    }

    /// Finds a child node for predictive search.
    ///
    /// Similar to find_child but also appends to key buffer.
    fn predictive_find_child(&self, agent: &mut crate::agent::Agent) -> bool {
        let state = agent.state().expect("Agent must have state");
        let query_pos = state.query_pos();
        let query_len = agent.query().length();

        assert!(query_pos < query_len, "Query position out of bounds");

        let node_id = state.node_id();
        let query_bytes = agent.query().as_bytes().to_vec();
        let query_char = query_bytes[query_pos];

        // Check cache first
        let cache_id = self.get_cache_id_with_label(node_id, query_char);
        if node_id == self.cache[cache_id].parent() {
            use crate::base::INVALID_EXTRA;
            if self.cache[cache_id].extra() != INVALID_EXTRA as usize {
                drop(state);
                if !self.prefix_match(agent, self.cache[cache_id].link()) {
                    return false;
                }
            } else {
                drop(state);
                let state = agent.state_mut().expect("Agent must have state");
                state.key_buf_mut().push(self.cache[cache_id].label());
                state.set_query_pos(query_pos + 1);
            }
            agent
                .state_mut()
                .expect("Agent must have state")
                .set_node_id(self.cache[cache_id].child());
            return true;
        }

        // Search children
        let mut louds_pos = self.louds.select0(node_id) + 1;
        if !self.louds.get(louds_pos) {
            return false;
        }

        let mut current_node = louds_pos - node_id - 1;
        drop(state);
        agent
            .state_mut()
            .expect("Agent must have state")
            .set_node_id(current_node);

        let mut link_id = crate::base::INVALID_LINK_ID as usize;

        loop {
            if self.link_flags.get(current_node) {
                link_id = self.update_link_id(link_id, current_node);
                let prev_query_pos = agent.state().expect("Agent must have state").query_pos();

                if self.prefix_match(agent, self.get_link_with_id(current_node, link_id)) {
                    return true;
                }

                if agent.state().expect("Agent must have state").query_pos() != prev_query_pos {
                    return false;
                }
            } else if self.bases[current_node] == query_char {
                let state = agent.state_mut().expect("Agent must have state");
                state.key_buf_mut().push(self.bases[current_node]);
                state.set_query_pos(query_pos + 1);
                return true;
            }

            current_node += 1;
            louds_pos += 1;
            agent
                .state_mut()
                .expect("Agent must have state")
                .set_node_id(current_node);

            if !self.louds.get(louds_pos) {
                break;
            }
        }

        false
    }

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
                if self.next_trie.is_some() {
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
