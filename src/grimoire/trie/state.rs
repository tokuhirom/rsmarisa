//! State type for trie operations.
//!
//! Ported from: lib/marisa/grimoire/trie/state.h
//!
//! State manages the internal state of a search agent during trie operations,
//! including node position, query position, history stack, and operation status.

use super::history::History;

/// Status codes for search operations.
///
/// These codes track the state of a search agent and what operations
/// are currently valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    /// Ready for any operation.
    ReadyToAll,
    /// Ready for common prefix search.
    ReadyToCommonPrefixSearch,
    /// Ready for predictive search.
    ReadyToPredictiveSearch,
    /// End of common prefix search results.
    EndOfCommonPrefixSearch,
    /// End of predictive search results.
    EndOfPredictiveSearch,
}

/// State for managing search operations in a trie.
///
/// State maintains the current position in the trie, query string,
/// and history of traversal for various search operations.
#[derive(Debug, Clone)]
pub struct State {
    /// Buffer for building/storing keys.
    key_buf: Vec<u8>,
    /// History stack for traversal.
    history: Vec<History>,
    /// Current node ID in the trie.
    node_id: u32,
    /// Current position in the query string.
    query_pos: u32,
    /// Current position in the history stack.
    history_pos: u32,
    /// Current operation status.
    status_code: StatusCode,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    /// Creates a new state with default values.
    pub fn new() -> Self {
        State {
            key_buf: Vec::new(),
            history: Vec::new(),
            node_id: 0,
            query_pos: 0,
            history_pos: 0,
            status_code: StatusCode::ReadyToAll,
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

    /// Sets the query position.
    ///
    /// # Arguments
    ///
    /// * `query_pos` - Query position to set
    ///
    /// # Panics
    ///
    /// Panics if query_pos > u32::MAX
    #[inline]
    pub fn set_query_pos(&mut self, query_pos: usize) {
        assert!(
            query_pos <= u32::MAX as usize,
            "Query position exceeds u32::MAX"
        );
        self.query_pos = query_pos as u32;
    }

    /// Sets the history position.
    ///
    /// # Arguments
    ///
    /// * `history_pos` - History position to set
    ///
    /// # Panics
    ///
    /// Panics if history_pos > u32::MAX
    #[inline]
    pub fn set_history_pos(&mut self, history_pos: usize) {
        assert!(
            history_pos <= u32::MAX as usize,
            "History position exceeds u32::MAX"
        );
        self.history_pos = history_pos as u32;
    }

    /// Sets the status code.
    #[inline]
    pub fn set_status_code(&mut self, status_code: StatusCode) {
        self.status_code = status_code;
    }

    /// Returns the node ID.
    #[inline]
    pub fn node_id(&self) -> usize {
        self.node_id as usize
    }

    /// Returns the query position.
    #[inline]
    pub fn query_pos(&self) -> usize {
        self.query_pos as usize
    }

    /// Returns the history position.
    #[inline]
    pub fn history_pos(&self) -> usize {
        self.history_pos as usize
    }

    /// Returns the status code.
    #[inline]
    pub fn status_code(&self) -> StatusCode {
        self.status_code
    }

    /// Returns an immutable reference to the key buffer.
    #[inline]
    pub fn key_buf(&self) -> &[u8] {
        &self.key_buf
    }

    /// Returns an immutable reference to the history stack.
    #[inline]
    pub fn history(&self) -> &[History] {
        &self.history
    }

    /// Returns a mutable reference to the key buffer.
    #[inline]
    pub fn key_buf_mut(&mut self) -> &mut Vec<u8> {
        &mut self.key_buf
    }

    /// Returns a mutable reference to the history stack.
    #[inline]
    pub fn history_mut(&mut self) -> &mut Vec<History> {
        &mut self.history
    }

    /// Returns the size of the history stack.
    #[inline]
    pub fn history_size(&self) -> usize {
        self.history.len()
    }

    /// Pushes a history entry onto the stack.
    #[inline]
    pub fn push_history(&mut self, history: History) {
        self.history.push(history);
    }

    /// Returns a reference to the last history entry.
    ///
    /// # Panics
    ///
    /// Panics if the history stack is empty.
    #[inline]
    pub fn history_back(&self) -> &History {
        self.history.last().expect("History stack is empty")
    }

    /// Returns a reference to the history entry at the given index.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    #[inline]
    pub fn history_at(&self, index: usize) -> &History {
        &self.history[index]
    }

    /// Returns a mutable reference to the history entry at the given index.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    #[inline]
    pub fn history_at_mut(&mut self, index: usize) -> &mut History {
        &mut self.history[index]
    }

    /// Resets the state to ready for any operation.
    pub fn reset(&mut self) {
        self.status_code = StatusCode::ReadyToAll;
    }

    /// Initializes state for lookup operation.
    pub fn lookup_init(&mut self) {
        self.node_id = 0;
        self.query_pos = 0;
        self.status_code = StatusCode::ReadyToAll;
    }

    /// Initializes state for reverse lookup operation.
    pub fn reverse_lookup_init(&mut self) {
        self.key_buf.clear();
        self.key_buf.reserve(32);
        self.status_code = StatusCode::ReadyToAll;
    }

    /// Initializes state for common prefix search operation.
    pub fn common_prefix_search_init(&mut self) {
        self.node_id = 0;
        self.query_pos = 0;
        self.status_code = StatusCode::ReadyToCommonPrefixSearch;
    }

    /// Initializes state for predictive search operation.
    pub fn predictive_search_init(&mut self) {
        self.key_buf.clear();
        self.key_buf.reserve(64);
        self.history.clear();
        self.history.reserve(4);
        self.node_id = 0;
        self.query_pos = 0;
        self.history_pos = 0;
        self.status_code = StatusCode::ReadyToPredictiveSearch;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_new() {
        let state = State::new();
        assert_eq!(state.node_id(), 0);
        assert_eq!(state.query_pos(), 0);
        assert_eq!(state.history_pos(), 0);
        assert_eq!(state.status_code(), StatusCode::ReadyToAll);
        assert_eq!(state.key_buf().len(), 0);
        assert_eq!(state.history().len(), 0);
    }

    #[test]
    fn test_state_default() {
        let state = State::default();
        assert_eq!(state.node_id(), 0);
        assert_eq!(state.query_pos(), 0);
        assert_eq!(state.history_pos(), 0);
        assert_eq!(state.status_code(), StatusCode::ReadyToAll);
    }

    #[test]
    fn test_state_set_node_id() {
        let mut state = State::new();
        state.set_node_id(100);
        assert_eq!(state.node_id(), 100);
    }

    #[test]
    fn test_state_set_query_pos() {
        let mut state = State::new();
        state.set_query_pos(50);
        assert_eq!(state.query_pos(), 50);
    }

    #[test]
    fn test_state_set_history_pos() {
        let mut state = State::new();
        state.set_history_pos(10);
        assert_eq!(state.history_pos(), 10);
    }

    #[test]
    fn test_state_set_status_code() {
        let mut state = State::new();
        state.set_status_code(StatusCode::ReadyToCommonPrefixSearch);
        assert_eq!(state.status_code(), StatusCode::ReadyToCommonPrefixSearch);
    }

    #[test]
    fn test_state_key_buf() {
        let mut state = State::new();
        state.key_buf_mut().push(b'h');
        state.key_buf_mut().push(b'i');

        assert_eq!(state.key_buf(), &[b'h', b'i']);
    }

    #[test]
    fn test_state_history() {
        let mut state = State::new();
        let mut hist = History::new();
        hist.set_node_id(10);
        state.history_mut().push(hist);

        assert_eq!(state.history().len(), 1);
        assert_eq!(state.history()[0].node_id(), 10);
    }

    #[test]
    fn test_state_reset() {
        let mut state = State::new();
        state.set_status_code(StatusCode::EndOfCommonPrefixSearch);
        state.reset();

        assert_eq!(state.status_code(), StatusCode::ReadyToAll);
    }

    #[test]
    fn test_state_lookup_init() {
        let mut state = State::new();
        state.set_node_id(100);
        state.set_query_pos(50);
        state.set_status_code(StatusCode::EndOfCommonPrefixSearch);

        state.lookup_init();

        assert_eq!(state.node_id(), 0);
        assert_eq!(state.query_pos(), 0);
        assert_eq!(state.status_code(), StatusCode::ReadyToAll);
    }

    #[test]
    fn test_state_reverse_lookup_init() {
        let mut state = State::new();
        state.key_buf_mut().push(b'x');
        state.key_buf_mut().push(b'y');

        state.reverse_lookup_init();

        assert_eq!(state.key_buf().len(), 0);
        assert!(state.key_buf_mut().capacity() >= 32);
        assert_eq!(state.status_code(), StatusCode::ReadyToAll);
    }

    #[test]
    fn test_state_common_prefix_search_init() {
        let mut state = State::new();
        state.set_node_id(100);
        state.set_query_pos(50);

        state.common_prefix_search_init();

        assert_eq!(state.node_id(), 0);
        assert_eq!(state.query_pos(), 0);
        assert_eq!(state.status_code(), StatusCode::ReadyToCommonPrefixSearch);
    }

    #[test]
    fn test_state_predictive_search_init() {
        let mut state = State::new();
        state.key_buf_mut().push(b'a');
        state.history_mut().push(History::new());
        state.set_node_id(100);
        state.set_query_pos(50);
        state.set_history_pos(10);

        state.predictive_search_init();

        assert_eq!(state.key_buf().len(), 0);
        assert!(state.key_buf_mut().capacity() >= 64);
        assert_eq!(state.history().len(), 0);
        assert!(state.history_mut().capacity() >= 4);
        assert_eq!(state.node_id(), 0);
        assert_eq!(state.query_pos(), 0);
        assert_eq!(state.history_pos(), 0);
        assert_eq!(state.status_code(), StatusCode::ReadyToPredictiveSearch);
    }

    #[test]
    fn test_state_clone() {
        let mut state1 = State::new();
        state1.set_node_id(42);
        state1.key_buf_mut().push(b't');

        let state2 = state1.clone();
        assert_eq!(state2.node_id(), 42);
        assert_eq!(state2.key_buf(), &[b't']);
    }

    #[test]
    fn test_status_code_equality() {
        assert_eq!(StatusCode::ReadyToAll, StatusCode::ReadyToAll);
        assert_ne!(
            StatusCode::ReadyToAll,
            StatusCode::ReadyToCommonPrefixSearch
        );
    }

    #[test]
    fn test_state_max_values() {
        let mut state = State::new();
        state.set_node_id(u32::MAX as usize);
        state.set_query_pos(u32::MAX as usize);
        state.set_history_pos(u32::MAX as usize);

        assert_eq!(state.node_id(), u32::MAX as usize);
        assert_eq!(state.query_pos(), u32::MAX as usize);
        assert_eq!(state.history_pos(), u32::MAX as usize);
    }
}
