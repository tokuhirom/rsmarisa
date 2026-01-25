//! Configuration for trie construction.
//!
//! Ported from: lib/marisa/grimoire/trie/config.h
//!
//! This module provides configuration parsing and management for trie building.
//! Configuration flags control various aspects of trie construction including
//! the number of tries, cache size, tail storage mode, and node ordering.

use crate::base::{CacheLevel, NodeOrder, NumTries, TailMode};

/// Configuration masks and constants.
mod masks {
    pub const NUM_TRIES_MASK: i32 = 0x0007F;
    pub const CACHE_LEVEL_MASK: i32 = 0x00F80;
    pub const TAIL_MODE_MASK: i32 = 0x0F000;
    pub const NODE_ORDER_MASK: i32 = 0xF0000;
    pub const CONFIG_MASK: i32 = 0xFFFFF;
}

/// Configuration for trie building.
///
/// Config parses and stores settings that control how tries are constructed.
/// These settings can be specified as bit flags and affect performance and
/// space characteristics of the resulting trie.
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// Number of tries to build (1-127).
    num_tries: usize,
    /// Cache level for construction.
    cache_level: CacheLevel,
    /// Tail storage mode (text or binary).
    tail_mode: TailMode,
    /// Node ordering (by label or weight).
    node_order: NodeOrder,
}

impl Config {
    /// Creates a new configuration with default values.
    pub fn new() -> Self {
        Config {
            num_tries: NumTries::DEFAULT as usize,
            cache_level: CacheLevel::default(),
            tail_mode: TailMode::default(),
            node_order: NodeOrder::default(),
        }
    }

    /// Parses configuration from flags.
    ///
    /// # Arguments
    ///
    /// * `config_flags` - Bit flags specifying configuration options
    ///
    /// # Panics
    ///
    /// Panics if invalid flags are provided (TODO: should return Result)
    pub fn parse(&mut self, config_flags: i32) {
        let mut temp = Config::new();
        temp.parse_internal(config_flags);
        self.swap(&mut temp);
    }

    /// Returns the configuration as bit flags.
    ///
    /// # Returns
    ///
    /// Integer containing all configuration settings as bit flags
    pub fn flags(&self) -> i32 {
        (self.num_tries as i32) | (self.tail_mode as i32) | (self.node_order as i32)
    }

    /// Returns the number of tries.
    #[inline]
    pub fn num_tries(&self) -> usize {
        self.num_tries
    }

    /// Returns the cache level.
    #[inline]
    pub fn cache_level(&self) -> CacheLevel {
        self.cache_level
    }

    /// Returns the tail mode.
    #[inline]
    pub fn tail_mode(&self) -> TailMode {
        self.tail_mode
    }

    /// Returns the node order.
    #[inline]
    pub fn node_order(&self) -> NodeOrder {
        self.node_order
    }

    /// Clears the configuration to default values.
    pub fn clear(&mut self) {
        *self = Config::new();
    }

    /// Swaps the contents of two configurations.
    pub fn swap(&mut self, other: &mut Config) {
        std::mem::swap(&mut self.num_tries, &mut other.num_tries);
        std::mem::swap(&mut self.cache_level, &mut other.cache_level);
        std::mem::swap(&mut self.tail_mode, &mut other.tail_mode);
        std::mem::swap(&mut self.node_order, &mut other.node_order);
    }

    /// Internal parsing implementation.
    fn parse_internal(&mut self, config_flags: i32) {
        assert!(
            (config_flags & !masks::CONFIG_MASK) == 0,
            "Invalid configuration flags"
        );

        self.parse_num_tries(config_flags);
        self.parse_cache_level(config_flags);
        self.parse_tail_mode(config_flags);
        self.parse_node_order(config_flags);
    }

    /// Parses the number of tries from flags.
    fn parse_num_tries(&mut self, config_flags: i32) {
        let num_tries = config_flags & masks::NUM_TRIES_MASK;
        if num_tries != 0 {
            self.num_tries = num_tries as usize;
        }
    }

    /// Parses the cache level from flags.
    fn parse_cache_level(&mut self, config_flags: i32) {
        let cache_level_bits = config_flags & masks::CACHE_LEVEL_MASK;

        self.cache_level = match cache_level_bits {
            0 => CacheLevel::default(),
            x if x == CacheLevel::Huge as i32 => CacheLevel::Huge,
            x if x == CacheLevel::Large as i32 => CacheLevel::Large,
            x if x == CacheLevel::Normal as i32 => CacheLevel::Normal,
            x if x == CacheLevel::Small as i32 => CacheLevel::Small,
            x if x == CacheLevel::Tiny as i32 => CacheLevel::Tiny,
            _ => panic!("Undefined cache level"),
        };
    }

    /// Parses the tail mode from flags.
    fn parse_tail_mode(&mut self, config_flags: i32) {
        let tail_mode_bits = config_flags & masks::TAIL_MODE_MASK;

        self.tail_mode = match tail_mode_bits {
            0 => TailMode::default(),
            x if x == TailMode::TextTail as i32 => TailMode::TextTail,
            x if x == TailMode::BinaryTail as i32 => TailMode::BinaryTail,
            _ => panic!("Undefined tail mode"),
        };
    }

    /// Parses the node order from flags.
    fn parse_node_order(&mut self, config_flags: i32) {
        let node_order_bits = config_flags & masks::NODE_ORDER_MASK;

        self.node_order = match node_order_bits {
            0 => NodeOrder::default(),
            x if x == NodeOrder::Label as i32 => NodeOrder::Label,
            x if x == NodeOrder::Weight as i32 => NodeOrder::Weight,
            _ => panic!("Undefined node order"),
        };
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert_eq!(config.num_tries(), NumTries::DEFAULT as usize);
        assert_eq!(config.cache_level() as i32, CacheLevel::default() as i32);
        assert_eq!(config.tail_mode() as i32, TailMode::default() as i32);
        assert_eq!(config.node_order() as i32, NodeOrder::default() as i32);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.num_tries(), NumTries::DEFAULT as usize);
    }

    #[test]
    fn test_config_parse_num_tries() {
        let mut config = Config::new();
        config.parse(10); // Set num_tries to 10
        assert_eq!(config.num_tries(), 10);
    }

    #[test]
    fn test_config_parse_cache_level() {
        let mut config = Config::new();
        config.parse(CacheLevel::Huge as i32);
        assert_eq!(config.cache_level() as i32, CacheLevel::Huge as i32);

        config.parse(CacheLevel::Tiny as i32);
        assert_eq!(config.cache_level() as i32, CacheLevel::Tiny as i32);
    }

    #[test]
    fn test_config_parse_tail_mode() {
        let mut config = Config::new();
        config.parse(TailMode::TextTail as i32);
        assert_eq!(config.tail_mode() as i32, TailMode::TextTail as i32);

        config.parse(TailMode::BinaryTail as i32);
        assert_eq!(config.tail_mode() as i32, TailMode::BinaryTail as i32);
    }

    #[test]
    fn test_config_parse_node_order() {
        let mut config = Config::new();
        config.parse(NodeOrder::Label as i32);
        assert_eq!(config.node_order() as i32, NodeOrder::Label as i32);

        config.parse(NodeOrder::Weight as i32);
        assert_eq!(config.node_order() as i32, NodeOrder::Weight as i32);
    }

    #[test]
    fn test_config_parse_combined() {
        let mut config = Config::new();
        let flags = 5 | (CacheLevel::Large as i32) | (TailMode::BinaryTail as i32) | (NodeOrder::Weight as i32);
        config.parse(flags);

        assert_eq!(config.num_tries(), 5);
        assert_eq!(config.cache_level() as i32, CacheLevel::Large as i32);
        assert_eq!(config.tail_mode() as i32, TailMode::BinaryTail as i32);
        assert_eq!(config.node_order() as i32, NodeOrder::Weight as i32);
    }

    #[test]
    fn test_config_flags() {
        let mut config = Config::new();
        config.parse(5 | (TailMode::TextTail as i32) | (NodeOrder::Label as i32));

        let flags = config.flags();
        assert_eq!(flags & masks::NUM_TRIES_MASK, 5);
        assert_eq!(flags & masks::TAIL_MODE_MASK, TailMode::TextTail as i32);
        assert_eq!(flags & masks::NODE_ORDER_MASK, NodeOrder::Label as i32);
    }

    #[test]
    fn test_config_clear() {
        let mut config = Config::new();
        config.parse(10);
        assert_eq!(config.num_tries(), 10);

        config.clear();
        assert_eq!(config.num_tries(), NumTries::DEFAULT as usize);
    }

    #[test]
    fn test_config_swap() {
        let mut config1 = Config::new();
        let mut config2 = Config::new();

        config1.parse(5);
        config2.parse(10);

        assert_eq!(config1.num_tries(), 5);
        assert_eq!(config2.num_tries(), 10);

        config1.swap(&mut config2);

        assert_eq!(config1.num_tries(), 10);
        assert_eq!(config2.num_tries(), 5);
    }

    #[test]
    #[should_panic(expected = "Invalid configuration flags")]
    fn test_config_parse_invalid_flags() {
        let mut config = Config::new();
        config.parse(0xFFFFFFF); // Invalid flags outside CONFIG_MASK
    }

    #[test]
    #[should_panic(expected = "Undefined cache level")]
    fn test_config_parse_invalid_cache() {
        let mut config = Config::new();
        config.parse(0x00900); // Invalid cache level (combines multiple bits)
    }

    #[test]
    #[should_panic(expected = "Undefined tail mode")]
    fn test_config_parse_invalid_tail() {
        let mut config = Config::new();
        config.parse(0x0C000); // Invalid tail mode
    }

    #[test]
    #[should_panic(expected = "Undefined node order")]
    fn test_config_parse_invalid_order() {
        let mut config = Config::new();
        config.parse(0xC0000); // Invalid node order
    }
}
