//! Configuration for trie construction.
//!
//! Ported from: lib/marisa/grimoire/trie/config.h

use crate::base::{CacheLevel, NodeOrder, NumTries, TailMode};

/// Configuration for trie building.
#[derive(Debug, Clone, Copy)]
pub struct Config {
    num_tries: u32,
    cache_level: CacheLevel,
    tail_mode: TailMode,
    node_order: NodeOrder,
}

impl Config {
    /// Creates a new configuration.
    pub fn new() -> Self {
        Config {
            num_tries: NumTries::DEFAULT,
            cache_level: CacheLevel::default(),
            tail_mode: TailMode::default(),
            node_order: NodeOrder::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
