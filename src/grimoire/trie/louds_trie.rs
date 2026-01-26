//! LOUDS (Level-Order Unary Degree Sequence) trie implementation.
//!
//! Ported from:
//! - lib/marisa/grimoire/trie/louds-trie.h
//! - lib/marisa/grimoire/trie/louds-trie.cc
//!
//! This is the core trie data structure using LOUDS encoding for
//! space-efficient storage while maintaining fast search operations.

use crate::base::{CacheLevel, NodeOrder, TailMode};
use crate::grimoire::io::{Mapper, Reader, Writer};
use crate::grimoire::trie::cache::Cache;
use crate::grimoire::trie::config::Config;
use crate::grimoire::trie::key::{Key, ReverseKey};
use crate::grimoire::trie::tail::Tail;
use crate::grimoire::vector::bit_vector::BitVector;
use crate::grimoire::vector::flat_vector::FlatVector;
use crate::grimoire::vector::vector::Vector;

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
    /// Mapper for memory-mapped access (not yet implemented).
    #[allow(dead_code)]
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

    /// Debug: returns if a node is terminal (for testing).
    #[cfg(test)]
    pub fn is_terminal(&self, node_id: usize) -> bool {
        self.terminal_flags.get(node_id)
    }

    /// Debug: returns if a node has a link (for testing).
    #[cfg(test)]
    pub fn has_link(&self, node_id: usize) -> bool {
        node_id < self.link_flags.size() && self.link_flags.get(node_id)
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
        use crate::grimoire::trie::header::Header;
        use std::mem::size_of;

        let mut size = Header::new().io_size()
            + self.louds.io_size()
            + self.terminal_flags.io_size()
            + self.link_flags.io_size()
            + self.bases.io_size()
            + self.extras.io_size()
            + self.tail.io_size();

        // Add next_trie size (excluding its header to avoid double-counting)
        if let Some(ref next) = self.next_trie {
            size += next.io_size() - Header::new().io_size();
        }

        // Add cache size and two uint32 values (num_l1_nodes, config)
        size += self.cache.io_size() + (size_of::<u32>() * 2);

        size
    }

    /// Clears the trie to empty state.
    pub fn clear(&mut self) {
        *self = LoudsTrie::new();
    }

    /// Swaps with another trie.
    pub fn swap(&mut self, other: &mut LoudsTrie) {
        std::mem::swap(self, other);
    }

    /// Builds the trie from a keyset.
    ///
    /// # Arguments
    ///
    /// * `keyset` - Mutable keyset containing keys to build from
    /// * `flags` - Configuration flags
    pub fn build(&mut self, keyset: &mut crate::keyset::Keyset, flags: i32) {
        use crate::grimoire::trie::config::Config;

        let mut config = Config::new();
        config.parse(flags);

        let mut temp = LoudsTrie::new();
        temp.build_(keyset, &config);
        self.swap(&mut temp);
    }

    /// Internal build implementation.
    fn build_(&mut self, keyset: &mut crate::keyset::Keyset, config: &Config) {
        use crate::grimoire::trie::key::Key;
        use crate::grimoire::vector::vector::Vector;

        // Copy keys from keyset to Vector<Key>
        let mut keys: Vector<Key<'_>> = Vector::new();
        keys.resize(keyset.size(), Key::new());
        for i in 0..keyset.size() {
            let keyset_key = keyset.get(i);
            keys[i].set_str(keyset_key.as_bytes());
            keys[i].set_weight(keyset_key.weight());
        }

        // Build the trie structure
        let mut terminals: Vector<u32> = Vector::new();
        self.build_trie_key(&mut keys, &mut terminals, config, 1);

        // Build terminal flags from sorted terminal positions
        // Pairs of (node_id, original_index)
        let mut pairs: Vec<(u32, u32)> = Vec::new();
        for i in 0..terminals.size() {
            pairs.push((terminals[i], i as u32));
        }
        pairs.sort_by_key(|p| p.0);

        // Create terminal flags bit vector
        let mut node_id = 0;
        for &(terminal_node, _) in &pairs {
            while node_id < terminal_node {
                self.terminal_flags.push_back(false);
                node_id += 1;
            }
            if node_id == terminal_node {
                self.terminal_flags.push_back(true);
                node_id += 1;
            }
        }
        while node_id < self.bases.size() as u32 {
            self.terminal_flags.push_back(false);
            node_id += 1;
        }
        self.terminal_flags.push_back(false);
        self.terminal_flags.build(false, true);

        // Update keyset with final key IDs
        for &(terminal_node, original_idx) in &pairs {
            let key_id = self.terminal_flags.rank1(terminal_node as usize);
            keyset.get_mut(original_idx as usize).set_id(key_id);
        }
    }

    /// Builds a trie level with Key type.
    fn build_trie_key<'a>(
        &mut self,
        keys: &mut Vector<Key<'a>>,
        terminals: &mut Vector<u32>,
        config: &Config,
        trie_id: usize,
    ) {
        self.build_current_trie_key(keys, terminals, config, trie_id);

        let mut next_terminals: Vector<u32> = Vector::new();
        if !keys.empty() {
            self.build_next_trie_key(keys, &mut next_terminals, config, trie_id);
        }

        // Configure based on what was built
        if let Some(next) = &self.next_trie {
            let flags = ((next.num_tries() + 1) as i32)
                | (next.tail_mode() as i32)
                | (next.node_order() as i32);
            self.config.parse(flags);
        } else {
            let flags = 1
                | (self.tail.mode() as i32)
                | (config.node_order() as i32)
                | (config.cache_level() as i32);
            self.config.parse(flags);
        }

        // Build link flags
        self.link_flags.build(false, false);

        // Set bases and extras for links
        let mut node_id = 0;
        for i in 0..next_terminals.size() {
            while !self.link_flags.get(node_id) {
                node_id += 1;
            }
            self.bases[node_id] = (next_terminals[i] % 256) as u8;
            next_terminals[i] /= 256;
            node_id += 1;
        }
        self.extras.build(&next_terminals);

        self.fill_cache();
    }

    /// Builds the current trie level with Key type.
    fn build_current_trie_key<'a>(
        &mut self,
        keys: &mut Vector<Key<'a>>,
        terminals: &mut Vector<u32>,
        config: &Config,
        trie_id: usize,
    ) {
        use crate::grimoire::algorithm::sort;
        use crate::grimoire::trie::range::{make_range, make_weighted_range, Range, WeightedRange};
        use std::collections::VecDeque;

        // Set IDs for sorting
        for i in 0..keys.size() {
            keys[i].set_id(i);
        }

        // Sort keys
        let num_keys = {
            let key_slice = keys.as_mut_slice();
            sort::sort(key_slice)
        };
        self.reserve_cache(config, trie_id, num_keys);

        // Initialize LOUDS with root
        self.louds.push_back(true);
        self.louds.push_back(false);
        self.bases.push_back(0);
        self.link_flags.push_back(false);

        let mut queue: VecDeque<Range> = VecDeque::new();
        let mut w_ranges: Vec<WeightedRange> = Vec::new();

        // Store raw pointers to avoid borrow checker issues
        // The actual data lives in the original keyset (lifetime 'a)
        let mut next_key_data: Vec<(*const [u8], f32)> = Vec::new(); // (ptr to bytes, weight)

        queue.push_back(make_range(0, keys.size(), 0));

        while let Some(mut range) = queue.pop_front() {
            // Note: We calculate node_id based on queue size *after* pop,
            // but we need to add 1 to account for the element we just popped
            let node_id = self.link_flags.size() - queue.len() - 1;

            // Mark terminals at this position
            while range.begin() < range.end() && keys[range.begin()].length() == range.key_pos() {
                keys[range.begin()].set_terminal(node_id);
                range.set_begin(range.begin() + 1);
            }

            if range.begin() == range.end() {
                self.louds.push_back(false);
                continue;
            }

            // Group by first character, accumulating weights
            w_ranges.clear();
            let mut weight = keys[range.begin()].weight() as f64;
            let mut group_start = range.begin();

            for i in (range.begin() + 1)..range.end() {
                if range.key_pos() >= keys[i - 1].length()
                    || range.key_pos() >= keys[i].length()
                    || keys[i - 1].get(range.key_pos()) != keys[i].get(range.key_pos())
                {
                    w_ranges.push(make_weighted_range(
                        group_start,
                        i,
                        range.key_pos(),
                        weight as f32,
                    ));
                    group_start = i;
                    weight = 0.0;
                }
                weight += keys[i].weight() as f64;
            }
            w_ranges.push(make_weighted_range(
                group_start,
                range.end(),
                range.key_pos(),
                weight as f32,
            ));

            // Sort by weight if configured
            if config.node_order() == crate::base::NodeOrder::Weight {
                w_ranges.sort_by(|a, b| b.cmp(a)); // Descending order
            }

            // Track level 1 node count
            if node_id == 0 {
                self.num_l1_nodes = w_ranges.len();
            }

            // Process each group
            for w_range in &mut w_ranges {
                // Find common prefix length
                let mut key_pos = w_range.key_pos() + 1;
                while key_pos < keys[w_range.begin()].length() {
                    let mut all_same = true;
                    for j in (w_range.begin() + 1)..w_range.end() {
                        if key_pos >= keys[j - 1].length()
                            || key_pos >= keys[j].length()
                            || keys[j - 1].get(key_pos) != keys[j].get(key_pos)
                        {
                            all_same = false;
                            break;
                        }
                    }
                    if !all_same {
                        break;
                    }
                    key_pos += 1;
                }

                // Add to cache (stub - will implement later)
                let label = keys[w_range.begin()].get(w_range.key_pos());
                self.cache_entry(node_id, self.bases.size(), w_range.weight(), label);

                if key_pos == w_range.key_pos() + 1 {
                    // Single character - store in bases
                    self.bases.push_back(label);
                    self.link_flags.push_back(false);
                } else {
                    // Multi-character - store pointer for creating next_keys later
                    self.bases.push_back(0);
                    self.link_flags.push_back(true);

                    let start = w_range.key_pos();
                    let len = key_pos - w_range.key_pos();
                    let key_bytes = keys[w_range.begin()].as_bytes();
                    let substring = &key_bytes[start..start + len];
                    // Store raw pointer to avoid borrow checker issues
                    // SAFETY: The slice is valid for lifetime 'a (from original keyset)
                    let ptr: *const [u8] = substring as *const [u8];
                    next_key_data.push((ptr, w_range.weight()));
                }

                w_range.set_key_pos(key_pos);
                queue.push_back(*w_range.range());
                self.louds.push_back(true);
            }
            self.louds.push_back(false);
        }

        self.louds.push_back(false);
        self.louds.build(trie_id == 1, true);
        self.bases.shrink();

        self.build_terminals_key(keys, terminals);

        // Now clear keys and populate with new Keys from stored pointers
        keys.clear();
        for (ptr, weight) in next_key_data {
            // SAFETY: The pointer is valid for lifetime 'a (from original keyset)
            // and keys vector lifetime doesn't affect the validity of the original data
            let substring: &[u8] = unsafe { &*ptr };

            let mut next_key = Key::new();
            next_key.set_str(substring);
            next_key.set_weight(weight);
            keys.push_back(next_key);
        }
    }

    /// Builds next trie or tail for Key type.
    fn build_next_trie_key<'a>(
        &mut self,
        keys: &mut Vector<Key<'a>>,
        terminals: &mut Vector<u32>,
        config: &Config,
        trie_id: usize,
    ) {
        use crate::grimoire::trie::entry::Entry;
        use crate::grimoire::trie::key::ReverseKey;

        if trie_id == config.num_tries() {
            // Build tail storage
            let mut entries: Vector<Entry<'_>> = Vector::new();
            entries.resize(keys.size(), Entry::new());
            for i in 0..keys.size() {
                entries[i].set_str(keys[i].as_bytes());
            }
            self.tail.build(&mut entries, terminals, config.tail_mode());
            return;
        }

        // Build next trie level with reversed keys
        // Collect data using raw pointers to avoid borrow checker issues
        let reverse_key_data: Vec<(*const [u8], f32)> = (0..keys.size())
            .map(|i| {
                let bytes = keys[i].as_bytes();
                let ptr: *const [u8] = bytes as *const [u8];
                (ptr, keys[i].weight())
            })
            .collect();

        keys.clear();

        let mut reverse_keys: Vector<ReverseKey<'_>> = Vector::new();
        for (ptr, weight) in reverse_key_data {
            // SAFETY: The pointer is valid for lifetime 'a (from original keyset)
            let bytes: &[u8] = unsafe { &*ptr };
            let mut rev_key = ReverseKey::new();
            rev_key.set_str(bytes);
            rev_key.set_weight(weight);
            reverse_keys.push_back(rev_key);
        }

        self.next_trie = Some(Box::new(LoudsTrie::new()));
        self.next_trie.as_mut().unwrap().build_trie_reverse(
            &mut reverse_keys,
            terminals,
            config,
            trie_id + 1,
        );
    }

    /// Builds a trie level with ReverseKey type.
    fn build_trie_reverse<'a>(
        &mut self,
        keys: &mut Vector<ReverseKey<'a>>,
        terminals: &mut Vector<u32>,
        config: &Config,
        trie_id: usize,
    ) {
        self.build_current_trie_reverse(keys, terminals, config, trie_id);

        let mut next_terminals: Vector<u32> = Vector::new();
        if !keys.empty() {
            self.build_next_trie_reverse(keys, &mut next_terminals, config, trie_id);
        }

        // Configure based on what was built
        if let Some(next) = &self.next_trie {
            let flags = ((next.num_tries() + 1) as i32)
                | (next.tail_mode() as i32)
                | (next.node_order() as i32);
            self.config.parse(flags);
        } else {
            let flags = 1
                | (self.tail.mode() as i32)
                | (config.node_order() as i32)
                | (config.cache_level() as i32);
            self.config.parse(flags);
        }

        // Build link flags
        self.link_flags.build(false, false);

        // Set bases and extras for links
        let mut node_id = 0;
        for i in 0..next_terminals.size() {
            while !self.link_flags.get(node_id) {
                node_id += 1;
            }
            self.bases[node_id] = (next_terminals[i] % 256) as u8;
            next_terminals[i] /= 256;
            node_id += 1;
        }
        self.extras.build(&next_terminals);

        self.fill_cache();
    }

    /// Builds the current trie level with ReverseKey type.
    fn build_current_trie_reverse<'a>(
        &mut self,
        keys: &mut Vector<ReverseKey<'a>>,
        terminals: &mut Vector<u32>,
        config: &Config,
        trie_id: usize,
    ) {
        use crate::grimoire::algorithm::sort;
        use crate::grimoire::trie::range::{make_range, make_weighted_range, Range, WeightedRange};
        use std::collections::VecDeque;

        // Set IDs for sorting
        for i in 0..keys.size() {
            keys[i].set_id(i);
        }

        // Sort keys
        let num_keys = {
            let key_slice = keys.as_mut_slice();
            sort::sort(key_slice)
        };
        self.reserve_cache(config, trie_id, num_keys);

        // Initialize LOUDS with root
        self.louds.push_back(true);
        self.louds.push_back(false);
        self.bases.push_back(0);
        self.link_flags.push_back(false);

        // Store raw pointers to avoid borrow checker issues
        let mut next_key_data: Vec<(*const [u8], f32)> = Vec::new();

        let mut queue: VecDeque<Range> = VecDeque::new();
        let mut w_ranges: Vec<WeightedRange> = Vec::new();

        queue.push_back(make_range(0, keys.size(), 0));

        while let Some(mut range) = queue.pop_front() {
            // Note: We calculate node_id based on queue size *after* pop,
            // but we need to add 1 to account for the element we just popped
            let node_id = self.link_flags.size() - queue.len() - 1;

            // Mark terminals at this position
            while range.begin() < range.end() && keys[range.begin()].length() == range.key_pos() {
                keys[range.begin()].set_terminal(node_id);
                range.set_begin(range.begin() + 1);
            }

            if range.begin() == range.end() {
                self.louds.push_back(false);
                continue;
            }

            // Group by first character, accumulating weights
            w_ranges.clear();
            let mut weight = keys[range.begin()].weight() as f64;
            let mut group_start = range.begin();

            for i in (range.begin() + 1)..range.end() {
                if range.key_pos() >= keys[i - 1].length()
                    || range.key_pos() >= keys[i].length()
                    || keys[i - 1].get(range.key_pos()) != keys[i].get(range.key_pos())
                {
                    w_ranges.push(make_weighted_range(
                        group_start,
                        i,
                        range.key_pos(),
                        weight as f32,
                    ));
                    group_start = i;
                    weight = 0.0;
                }
                weight += keys[i].weight() as f64;
            }
            w_ranges.push(make_weighted_range(
                group_start,
                range.end(),
                range.key_pos(),
                weight as f32,
            ));

            // Sort by weight if configured
            if config.node_order() == crate::base::NodeOrder::Weight {
                w_ranges.sort_by(|a, b| b.cmp(a)); // Descending order
            }

            // Track level 1 node count
            if node_id == 0 {
                self.num_l1_nodes = w_ranges.len();
            }

            // Process each group
            for w_range in &mut w_ranges {
                // Find common prefix length
                let mut key_pos = w_range.key_pos() + 1;
                while key_pos < keys[w_range.begin()].length() {
                    let mut all_same = true;
                    for j in (w_range.begin() + 1)..w_range.end() {
                        if key_pos >= keys[j - 1].length()
                            || key_pos >= keys[j].length()
                            || keys[j - 1].get(key_pos) != keys[j].get(key_pos)
                        {
                            all_same = false;
                            break;
                        }
                    }
                    if !all_same {
                        break;
                    }
                    key_pos += 1;
                }

                // Add to cache (for ReverseKey, use get_cache_id without label)
                self.cache_entry_reverse(node_id, self.bases.size(), w_range.weight());

                if key_pos == w_range.key_pos() + 1 {
                    // Single character - store in bases
                    let label = keys[w_range.begin()].get(w_range.key_pos());
                    self.bases.push_back(label);
                    self.link_flags.push_back(false);
                } else {
                    // Multi-character - store pointer for creating next_keys later
                    self.bases.push_back(0);
                    self.link_flags.push_back(true);

                    let start = w_range.key_pos();
                    let len = key_pos - w_range.key_pos();
                    let key_bytes = keys[w_range.begin()].as_bytes();
                    // For ReverseKey: as_bytes() returns forward bytes, but start/len
                    // are indices in the reversed access order. We need to convert
                    // reverse indices to forward indices.
                    // If ReverseKey("banana") and start=3, len=3, we want "ban" not "ana"
                    let forward_start = key_bytes.len() - start - len;
                    let forward_end = key_bytes.len() - start;
                    let substring = &key_bytes[forward_start..forward_end];
                    // Store raw pointer to avoid borrow checker issues
                    // SAFETY: The slice is valid for lifetime 'a (from original keyset)
                    let ptr: *const [u8] = substring as *const [u8];
                    next_key_data.push((ptr, w_range.weight()));
                }

                w_range.set_key_pos(key_pos);
                queue.push_back(*w_range.range());
                self.louds.push_back(true);
            }
            self.louds.push_back(false);
        }

        self.louds.push_back(false);
        self.louds.build(trie_id == 1, true);
        self.bases.shrink();

        self.build_terminals_reverse(keys, terminals);

        // Now clear keys and populate with new Keys from stored pointers
        keys.clear();
        for (ptr, weight) in next_key_data {
            // SAFETY: The pointer is valid for lifetime 'a (from original keyset)
            let substring: &[u8] = unsafe { &*ptr };

            let mut next_key = ReverseKey::new();
            next_key.set_str(substring);
            next_key.set_weight(weight);
            keys.push_back(next_key);
        }
    }

    /// Builds next trie or tail for ReverseKey type.
    fn build_next_trie_reverse<'a>(
        &mut self,
        keys: &mut Vector<ReverseKey<'a>>,
        terminals: &mut Vector<u32>,
        config: &Config,
        trie_id: usize,
    ) {
        use crate::grimoire::trie::entry::Entry;

        if trie_id == config.num_tries() {
            // Build tail storage
            let mut entries: Vector<Entry<'_>> = Vector::new();
            entries.resize(keys.size(), Entry::new());
            for i in 0..keys.size() {
                entries[i].set_str(keys[i].as_bytes());
            }
            self.tail.build(&mut entries, terminals, config.tail_mode());
            return;
        }

        // Build next trie level (shouldn't happen for reverse keys in practice)
        self.next_trie = Some(Box::new(LoudsTrie::new()));
        self.next_trie
            .as_mut()
            .unwrap()
            .build_trie_reverse(keys, terminals, config, trie_id + 1);
    }

    /// Collects terminal positions from reverse keys.
    fn build_terminals_reverse<'a>(
        &self,
        keys: &Vector<ReverseKey<'a>>,
        terminals: &mut Vector<u32>,
    ) {
        let mut temp: Vector<u32> = Vector::new();
        temp.resize(keys.size(), 0);
        for i in 0..keys.size() {
            temp[keys[i].id()] = keys[i].terminal() as u32;
        }
        terminals.swap(&mut temp);
    }

    /// Adds a cache entry for ReverseKey type.
    fn cache_entry_reverse(&mut self, _parent: usize, child: usize, weight: f32) {
        let cache_id = self.get_cache_id(child);
        if weight > self.cache[cache_id].weight() {
            self.cache[cache_id].set_parent(_parent);
            self.cache[cache_id].set_child(child);
            self.cache[cache_id].set_weight(weight);
        }
    }

    /// Collects terminal positions from keys.
    fn build_terminals_key<'a>(&self, keys: &Vector<Key<'a>>, terminals: &mut Vector<u32>) {
        let mut temp: Vector<u32> = Vector::new();
        temp.resize(keys.size(), 0);
        for i in 0..keys.size() {
            temp[keys[i].id()] = keys[i].terminal() as u32;
        }
        terminals.swap(&mut temp);
    }

    /// Reserves cache based on configuration.
    fn reserve_cache(&mut self, config: &Config, trie_id: usize, num_keys: usize) {
        // Cache level value is the divisor
        let cache_level = config.cache_level() as i32 as usize;

        let mut cache_size = if trie_id == 1 { 256 } else { 1 };
        while cache_size < (num_keys / cache_level) {
            cache_size *= 2;
        }

        self.cache.resize(cache_size, Cache::new());
        self.cache_mask = cache_size - 1;
    }

    /// Adds a cache entry for Key type.
    fn cache_entry(&mut self, parent: usize, child: usize, weight: f32, label: u8) {
        assert!(parent < child, "Parent must be less than child");

        let cache_id = self.get_cache_id_with_label(parent, label);
        if weight > self.cache[cache_id].weight() {
            self.cache[cache_id].set_parent(parent);
            self.cache[cache_id].set_child(child);
            self.cache[cache_id].set_weight(weight);
        }
    }

    /// Fills the cache after building.
    fn fill_cache(&mut self) {
        use crate::base::INVALID_EXTRA;

        for i in 0..self.cache.size() {
            let node_id = self.cache[i].child();
            if node_id != 0 {
                self.cache[i].set_base(self.bases[node_id]);
                if !self.link_flags.get(node_id) {
                    self.cache[i].set_extra(INVALID_EXTRA as usize);
                } else {
                    let link_id = self.link_flags.rank1(node_id);
                    // Check if extras has been built and has the required index
                    if link_id < self.extras.size() {
                        self.cache[i].set_extra(self.extras.get(link_id) as usize);
                    } else {
                        self.cache[i].set_extra(INVALID_EXTRA as usize);
                    }
                }
            } else {
                self.cache[i].set_parent(u32::MAX as usize);
                self.cache[i].set_child(u32::MAX as usize);
            }
        }
    }

    /// Maps the trie from memory (stub).
    ///
    /// # Arguments
    ///
    /// * `_mapper` - Mapper for memory-mapped access
    ///
    /// TODO: Implement when BitVector, FlatVector, Tail have proper I/O support
    #[allow(dead_code)]
    pub fn map(&mut self, _mapper: &mut Mapper<'_>) {
        // Stub - requires proper I/O support in all components
    }

    /// Reads the trie from a reader (with header).
    ///
    /// # Arguments
    ///
    /// * `reader` - Reader to read from
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails or header is invalid
    pub fn read(&mut self, reader: &mut Reader) -> std::io::Result<()> {
        use crate::grimoire::trie::header::Header;
        Header::new().read(reader)?;
        self.read_internal(reader)
    }

    /// Writes the trie to a writer (with header).
    ///
    /// # Arguments
    ///
    /// * `writer` - Writer to write to
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails
    pub fn write(&self, writer: &mut Writer) -> std::io::Result<()> {
        use crate::grimoire::trie::header::Header;
        Header::new().write(writer)?;
        self.write_internal(writer)
    }

    /// Reads the trie from a reader (internal version without header).
    ///
    /// Format:
    /// - louds: BitVector
    /// - terminal_flags: BitVector
    /// - link_flags: BitVector
    /// - bases: `Vector<u8>`
    /// - extras: FlatVector
    /// - tail: Tail
    /// - next_trie: Optional recursive LoudsTrie (if link_flags.num_1s() != 0 && tail.empty())
    /// - cache: `Vector<Cache>`
    /// - num_l1_nodes: u32
    /// - config_flags: u32
    ///
    /// # Arguments
    ///
    /// * `reader` - Reader to read from
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails
    fn read_internal(&mut self, reader: &mut Reader) -> std::io::Result<()> {
        // Read all component data structures
        self.louds.read(reader)?;
        self.terminal_flags.read(reader)?;
        self.link_flags.read(reader)?;
        self.bases.read(reader)?;
        self.extras.read(reader)?;
        self.tail.read(reader)?;

        // Check if next_trie should exist
        if self.link_flags.num_1s() != 0 && self.tail.empty() {
            let mut next = Box::new(LoudsTrie::new());
            next.read_internal(reader)?;
            self.next_trie = Some(next);
        }

        // Read cache
        self.cache.read(reader)?;
        self.cache_mask = self.cache.size().saturating_sub(1);

        // Read num_l1_nodes
        let temp_num_l1_nodes: u32 = reader.read()?;
        self.num_l1_nodes = temp_num_l1_nodes as usize;

        // Read and parse config flags
        let temp_config_flags: u32 = reader.read()?;
        self.config.parse(temp_config_flags as i32);

        Ok(())
    }

    /// Writes the trie to a writer (internal version without header).
    ///
    /// Format:
    /// - louds: BitVector
    /// - terminal_flags: BitVector
    /// - link_flags: BitVector
    /// - bases: `Vector<u8>`
    /// - extras: FlatVector
    /// - tail: Tail
    /// - next_trie: Optional recursive LoudsTrie (if exists)
    /// - cache: `Vector<Cache>`
    /// - num_l1_nodes: u32
    /// - config_flags: u32
    ///
    /// # Arguments
    ///
    /// * `writer` - Writer to write to
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails
    fn write_internal(&self, writer: &mut Writer) -> std::io::Result<()> {
        // Write all component data structures
        self.louds.write(writer)?;
        self.terminal_flags.write(writer)?;
        self.link_flags.write(writer)?;
        self.bases.write(writer)?;
        self.extras.write(writer)?;
        self.tail.write(writer)?;

        // Write next_trie if it exists
        if let Some(ref next) = self.next_trie {
            next.write_internal(writer)?;
        }

        // Write cache
        self.cache.write(writer)?;

        // Write num_l1_nodes as u32
        writer.write(&(self.num_l1_nodes as u32))?;

        // Write config flags as u32
        writer.write(&(self.config.flags() as u32))?;

        Ok(())
    }

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

        // Set result key - point to the query buffer owned by agent
        let key_id = self.terminal_flags.rank1(node_id);
        agent.set_key_from_query();
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
            agent.set_key_from_state_buf();
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
                let _ = state;

                agent.set_key_from_state_buf();
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
                let _ = state;
                let state = agent.state_mut().expect("Agent must have state");
                state.common_prefix_search_init();

                // Check if root is terminal
                let node_id = state.node_id();
                if self.terminal_flags.get(node_id) {
                    let query_pos = state.query_pos();
                    let _ = state;

                    let key_id = self.terminal_flags.rank1(node_id);
                    agent.set_key_from_query_prefix(query_pos);
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
                let key_id = self.terminal_flags.rank1(node_id);
                agent.set_key_from_query_prefix(query_pos);
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
                let _ = state;
                let query_len = agent.query().length();
                let state = agent.state_mut().expect("Agent must have state");
                state.predictive_search_init();
                let _ = state;
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
                    let _ = state;

                    agent.set_key_from_state_buf();
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
                    *state.history_back()
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
                    let _ = state;

                    self.restore(agent, self.get_link_with_id(next_node_id, new_link_id));

                    let state = agent.state_mut().expect("Agent must have state");
                    let key_len = state.key_buf().len();
                    state.history_at_mut(history_pos).set_key_pos(key_len);
                } else {
                    state.key_buf_mut().push(self.bases[next_node_id]);
                    let key_len = state.key_buf().len();
                    state.history_at_mut(history_pos).set_key_pos(key_len);
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

                    let _ = state;

                    agent.set_key_from_state_buf();
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
                let _ = state;
                if !self.prefix_match(agent, self.cache[cache_id].link()) {
                    return false;
                }
            } else {
                let _ = state;
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
        let _ = state;
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
        // Defensive: Check if extras has the index (might be empty if tail not built yet)
        let extra = if extra_idx < self.extras.size() {
            self.extras.get(extra_idx) as usize
        } else {
            0
        };
        base | (extra * 256)
    }

    /// Gets link value from a node with specific link ID.
    #[inline]
    fn get_link_with_id(&self, node_id: usize, link_id: usize) -> usize {
        let base = self.bases[node_id] as usize;
        // Defensive: Check if extras has the index (might be empty if tail not built yet)
        let extra = if link_id < self.extras.size() {
            self.extras.get(link_id) as usize
        } else {
            0
        };
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
        let mut query_pos = agent.state().expect("Agent must have state").query_pos();

        let query_bytes = agent.query().as_bytes().to_vec();

        assert!(query_pos < query_len, "Query position out of bounds");
        assert!(node_id != 0, "Node ID must not be 0");

        let mut node_id = node_id;

        loop {
            let cache_id = self.get_cache_id(node_id);
            if node_id == self.cache[cache_id].child() {
                use crate::base::INVALID_EXTRA;
                if self.cache[cache_id].extra() != INVALID_EXTRA as usize {
                    if !self.match_link(agent, self.cache[cache_id].link()) {
                        return false;
                    }
                    // Re-sync local query_pos after match_link may have modified agent state
                    query_pos = agent.state().expect("Agent must have state").query_pos();
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
                    // Re-sync local query_pos after match_link may have modified agent state
                    query_pos = agent.state().expect("Agent must have state").query_pos();
                } else if !self.tail.match_tail(agent, self.get_link_simple(node_id)) {
                    return false;
                } else {
                    // Re-sync local query_pos after tail.match_tail may have modified agent state
                    query_pos = agent.state().expect("Agent must have state").query_pos();
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
        let mut query_pos = agent.state().expect("Agent must have state").query_pos();

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
                    // Re-sync local query_pos after prefix_match may have modified agent state
                    query_pos = agent.state().expect("Agent must have state").query_pos();
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
                    // Re-sync local query_pos after prefix_match may have modified agent state
                    query_pos = agent.state().expect("Agent must have state").query_pos();
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

    #[test]
    fn test_louds_trie_write_read_empty() {
        // Rust-specific: Test empty LoudsTrie serialization
        use crate::grimoire::io::{Reader, Writer};

        let trie = LoudsTrie::new();
        assert!(trie.empty());

        // Write to buffer
        let mut writer = Writer::from_vec(Vec::new());
        trie.write(&mut writer).unwrap();

        let data = writer.into_inner().unwrap();

        // Read back
        let mut reader = Reader::from_bytes(&data);
        let mut trie2 = LoudsTrie::new();
        trie2.read(&mut reader).unwrap();

        // Verify
        assert!(trie2.empty());
        assert_eq!(trie2.num_keys(), 0);
        assert_eq!(trie2.num_nodes(), 0);
    }

    #[test]
    fn test_louds_trie_write_read_with_keys() {
        // Rust-specific: Test LoudsTrie serialization with keys
        use crate::grimoire::io::{Reader, Writer};
        use crate::keyset::Keyset;

        // Build a trie with some keys
        let mut keyset = Keyset::new();
        keyset.push_back_str("app").unwrap();
        keyset.push_back_str("apple").unwrap();
        keyset.push_back_str("application").unwrap();

        let mut trie = LoudsTrie::new();
        trie.build(&mut keyset, 0);

        assert!(!trie.empty());
        assert_eq!(trie.num_keys(), 3);

        // Write to buffer
        let mut writer = Writer::from_vec(Vec::new());
        trie.write(&mut writer).unwrap();

        let data = writer.into_inner().unwrap();

        // Read back
        let mut reader = Reader::from_bytes(&data);
        let mut trie2 = LoudsTrie::new();
        trie2.read(&mut reader).unwrap();

        // Verify structure is preserved
        assert_eq!(trie2.num_keys(), 3);
        assert_eq!(trie2.num_nodes(), trie.num_nodes());
        assert_eq!(trie2.tail_mode(), trie.tail_mode());
        assert_eq!(trie2.node_order(), trie.node_order());

        // Verify all keys can still be looked up
        use crate::agent::Agent;

        let mut agent = Agent::new();
        agent.init_state().unwrap();

        agent.set_query_str("app");
        assert!(trie2.lookup(&mut agent));

        agent.set_query_str("apple");
        assert!(trie2.lookup(&mut agent));

        agent.set_query_str("application");
        assert!(trie2.lookup(&mut agent));

        // Non-existent key
        agent.set_query_str("apply");
        assert!(!trie2.lookup(&mut agent));
    }

    #[test]
    fn test_louds_trie_write_read_config_preserved() {
        // Rust-specific: Test that configuration is preserved through serialization
        use crate::base::{NodeOrder, TailMode};
        use crate::grimoire::io::{Reader, Writer};
        use crate::keyset::Keyset;

        // Build a trie with specific configuration
        let mut keyset = Keyset::new();
        keyset.push_back_str("test").unwrap();

        let mut trie = LoudsTrie::new();
        let flags = (TailMode::TextTail as i32) | (NodeOrder::Label as i32);
        trie.build(&mut keyset, flags);

        // Write to buffer
        let mut writer = Writer::from_vec(Vec::new());
        trie.write(&mut writer).unwrap();

        let data = writer.into_inner().unwrap();

        // Read back
        let mut reader = Reader::from_bytes(&data);
        let mut trie2 = LoudsTrie::new();
        trie2.read(&mut reader).unwrap();

        // Verify configuration is preserved
        assert_eq!(trie2.tail_mode(), TailMode::TextTail);
        assert_eq!(trie2.node_order(), NodeOrder::Label);
    }
}
