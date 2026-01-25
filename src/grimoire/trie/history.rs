//! History structure for trie traversal state.
//!
//! Ported from: lib/marisa/grimoire/trie/history.h
//!
//! History records the state at each step during trie traversal,
//! including node positions, key positions, and link information.

use crate::base::{INVALID_KEY_ID, INVALID_LINK_ID};

/// History entry recording trie traversal state.
///
/// History stores information about a position in the trie during
/// traversal, including the node ID, LOUDS position, key position,
/// link ID, and key ID.
#[derive(Debug, Clone, Copy)]
pub struct History {
    /// Node ID in the trie.
    node_id: u32,
    /// Position in LOUDS bit vector.
    louds_pos: u32,
    /// Position in the key string.
    key_pos: u32,
    /// Link ID (terminal or continuation).
    link_id: u32,
    /// Key ID.
    key_id: u32,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    /// Creates a new history with default values.
    pub fn new() -> Self {
        History {
            node_id: 0,
            louds_pos: 0,
            key_pos: 0,
            link_id: INVALID_LINK_ID,
            key_id: INVALID_KEY_ID,
        }
    }

    /// Sets the node ID.
    ///
    /// # Arguments
    ///
    /// * `node_id` - Node ID to set
    ///
    /// # Panics
    ///
    /// Panics if node_id > u32::MAX
    #[inline]
    pub fn set_node_id(&mut self, node_id: usize) {
        assert!(node_id <= u32::MAX as usize, "Node ID exceeds u32::MAX");
        self.node_id = node_id as u32;
    }

    /// Sets the LOUDS position.
    ///
    /// # Arguments
    ///
    /// * `louds_pos` - LOUDS position to set
    ///
    /// # Panics
    ///
    /// Panics if louds_pos > u32::MAX
    #[inline]
    pub fn set_louds_pos(&mut self, louds_pos: usize) {
        assert!(
            louds_pos <= u32::MAX as usize,
            "LOUDS position exceeds u32::MAX"
        );
        self.louds_pos = louds_pos as u32;
    }

    /// Sets the key position.
    ///
    /// # Arguments
    ///
    /// * `key_pos` - Key position to set
    ///
    /// # Panics
    ///
    /// Panics if key_pos > u32::MAX
    #[inline]
    pub fn set_key_pos(&mut self, key_pos: usize) {
        assert!(
            key_pos <= u32::MAX as usize,
            "Key position exceeds u32::MAX"
        );
        self.key_pos = key_pos as u32;
    }

    /// Sets the link ID.
    ///
    /// # Arguments
    ///
    /// * `link_id` - Link ID to set
    ///
    /// # Panics
    ///
    /// Panics if link_id > u32::MAX
    #[inline]
    pub fn set_link_id(&mut self, link_id: usize) {
        assert!(link_id <= u32::MAX as usize, "Link ID exceeds u32::MAX");
        self.link_id = link_id as u32;
    }

    /// Sets the key ID.
    ///
    /// # Arguments
    ///
    /// * `key_id` - Key ID to set
    ///
    /// # Panics
    ///
    /// Panics if key_id > u32::MAX
    #[inline]
    pub fn set_key_id(&mut self, key_id: usize) {
        assert!(key_id <= u32::MAX as usize, "Key ID exceeds u32::MAX");
        self.key_id = key_id as u32;
    }

    /// Returns the node ID.
    #[inline]
    pub fn node_id(&self) -> usize {
        self.node_id as usize
    }

    /// Returns the LOUDS position.
    #[inline]
    pub fn louds_pos(&self) -> usize {
        self.louds_pos as usize
    }

    /// Returns the key position.
    #[inline]
    pub fn key_pos(&self) -> usize {
        self.key_pos as usize
    }

    /// Returns the link ID.
    #[inline]
    pub fn link_id(&self) -> usize {
        self.link_id as usize
    }

    /// Returns the key ID.
    #[inline]
    pub fn key_id(&self) -> usize {
        self.key_id as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_new() {
        let history = History::new();
        assert_eq!(history.node_id(), 0);
        assert_eq!(history.louds_pos(), 0);
        assert_eq!(history.key_pos(), 0);
        assert_eq!(history.link_id(), INVALID_LINK_ID as usize);
        assert_eq!(history.key_id(), INVALID_KEY_ID as usize);
    }

    #[test]
    fn test_history_default() {
        let history = History::default();
        assert_eq!(history.node_id(), 0);
        assert_eq!(history.louds_pos(), 0);
        assert_eq!(history.key_pos(), 0);
        assert_eq!(history.link_id(), INVALID_LINK_ID as usize);
        assert_eq!(history.key_id(), INVALID_KEY_ID as usize);
    }

    #[test]
    fn test_history_set_node_id() {
        let mut history = History::new();
        history.set_node_id(100);
        assert_eq!(history.node_id(), 100);
    }

    #[test]
    fn test_history_set_louds_pos() {
        let mut history = History::new();
        history.set_louds_pos(200);
        assert_eq!(history.louds_pos(), 200);
    }

    #[test]
    fn test_history_set_key_pos() {
        let mut history = History::new();
        history.set_key_pos(50);
        assert_eq!(history.key_pos(), 50);
    }

    #[test]
    fn test_history_set_link_id() {
        let mut history = History::new();
        history.set_link_id(123);
        assert_eq!(history.link_id(), 123);
    }

    #[test]
    fn test_history_set_key_id() {
        let mut history = History::new();
        history.set_key_id(456);
        assert_eq!(history.key_id(), 456);
    }

    #[test]
    fn test_history_all_fields() {
        let mut history = History::new();
        history.set_node_id(10);
        history.set_louds_pos(20);
        history.set_key_pos(5);
        history.set_link_id(30);
        history.set_key_id(40);

        assert_eq!(history.node_id(), 10);
        assert_eq!(history.louds_pos(), 20);
        assert_eq!(history.key_pos(), 5);
        assert_eq!(history.link_id(), 30);
        assert_eq!(history.key_id(), 40);
    }

    #[test]
    fn test_history_max_values() {
        let mut history = History::new();
        history.set_node_id(u32::MAX as usize);
        history.set_louds_pos(u32::MAX as usize);
        history.set_key_pos(u32::MAX as usize);
        history.set_link_id(u32::MAX as usize);
        history.set_key_id(u32::MAX as usize);

        assert_eq!(history.node_id(), u32::MAX as usize);
        assert_eq!(history.louds_pos(), u32::MAX as usize);
        assert_eq!(history.key_pos(), u32::MAX as usize);
        assert_eq!(history.link_id(), u32::MAX as usize);
        assert_eq!(history.key_id(), u32::MAX as usize);
    }

    #[test]
    fn test_history_clone() {
        let mut history1 = History::new();
        history1.set_node_id(10);
        history1.set_key_pos(5);

        let history2 = history1;
        assert_eq!(history2.node_id(), 10);
        assert_eq!(history2.key_pos(), 5);
    }

    #[test]
    fn test_history_copy() {
        let mut history1 = History::new();
        history1.set_node_id(10);

        let history2 = history1;
        // Both should have the same value since it's Copy
        assert_eq!(history1.node_id(), 10);
        assert_eq!(history2.node_id(), 10);
    }
}
