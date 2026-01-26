//! Agent for trie operations.
//!
//! Ported from:
//! - include/marisa/agent.h
//! - lib/marisa/agent.cc

use crate::grimoire::trie::state::{State, StatusCode};
use crate::key::Key;
use crate::query::Query;
use std::io;

/// Agent encapsulates query, key, and state for trie operations.
///
/// An agent is used for:
/// - Lookup operations (query → key with ID)
/// - Reverse lookup (query with ID → key with string)
/// - Common prefix search
/// - Predictive search
pub struct Agent {
    /// Query for search operations.
    query: Query,
    /// Key result from operations.
    key: Key,
    /// Optional state for complex searches.
    state: Option<Box<State>>,
}

impl Default for Agent {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Agent {
    fn clone(&self) -> Self {
        let mut cloned = Agent {
            query: self.query.clone(),
            key: self.key.clone(),
            state: self.state.as_ref().map(|s| Box::new((**s).clone())),
        };

        // Update agent after copying state
        let should_update = cloned.state.is_some();
        if should_update {
            if let Some(ref state) = cloned.state {
                let status = state.status_code();
                match status {
                    StatusCode::ReadyToPredictiveSearch | StatusCode::EndOfPredictiveSearch => {
                        // Key points into state buffer - repoint after copy
                        let key_buf = state.key_buf();
                        cloned.key.set_bytes(key_buf);
                    }
                    _ => {
                        // Key is null or points to query - no update needed
                    }
                }
            }
        }

        cloned
    }
}

impl Agent {
    /// Creates a new empty agent.
    pub fn new() -> Self {
        Agent {
            query: Query::new(),
            key: Key::new(),
            state: None,
        }
    }

    /// Returns a reference to the query.
    pub fn query(&self) -> &Query {
        &self.query
    }

    /// Returns a mutable reference to the query.
    pub fn query_mut(&mut self) -> &mut Query {
        &mut self.query
    }

    /// Returns a reference to the key.
    pub fn key(&self) -> &Key {
        &self.key
    }

    /// Returns a mutable reference to the key.
    pub fn key_mut(&mut self) -> &mut Key {
        &mut self.key
    }

    /// Sets the query from a string slice.
    pub fn set_query_str(&mut self, s: &str) {
        if let Some(ref mut state) = self.state {
            state.reset();
        }
        self.query.set_str(s);
    }

    /// Sets the query from a byte slice.
    pub fn set_query_bytes(&mut self, bytes: &[u8]) {
        if let Some(ref mut state) = self.state {
            state.reset();
        }
        self.query.set_bytes(bytes);
    }

    /// Sets the query from a key ID for reverse lookup.
    pub fn set_query_id(&mut self, key_id: usize) {
        if let Some(ref mut state) = self.state {
            state.reset();
        }
        self.query.set_id(key_id);
    }

    /// Returns a reference to the state if it exists.
    pub fn state(&self) -> Option<&State> {
        self.state.as_deref()
    }

    /// Returns a mutable reference to the state if it exists.
    pub fn state_mut(&mut self) -> Option<&mut State> {
        self.state.as_deref_mut()
    }

    /// Sets the key from a string slice.
    pub fn set_key_str(&mut self, s: &str) {
        self.key.set_str(s);
    }

    /// Sets the key from a byte slice.
    pub fn set_key_bytes(&mut self, bytes: &[u8]) {
        self.key.set_bytes(bytes);
    }

    /// Sets the key ID.
    pub fn set_key_id(&mut self, id: usize) {
        self.key.set_id(id);
    }

    /// Sets the key to point to the state's key buffer.
    ///
    /// This is used after operations like reverse_lookup that build
    /// the key in the state's buffer.
    pub fn set_key_from_state_buf(&mut self) {
        if let Some(ref state) = self.state {
            let buf = state.key_buf();
            // Set key to point to state's buffer
            // SAFETY: The buffer is owned by state which is owned by self,
            // so it will live as long as self lives.
            self.key.set_bytes(buf);
        } else {
            panic!("Agent must have state to set key from state buffer");
        }
    }

    /// Sets the key to point to the query buffer.
    ///
    /// This is used after a successful lookup to set the result key
    /// to the query string that was searched for.
    pub fn set_key_from_query(&mut self) {
        let bytes = self.query.as_bytes();
        self.key.set_bytes(bytes);
    }

    /// Sets the key to point to a prefix of the query buffer.
    ///
    /// This is used during prefix searches to set the result key
    /// to a prefix of the query string.
    ///
    /// # Arguments
    ///
    /// * `length` - Length of the prefix
    pub fn set_key_from_query_prefix(&mut self, length: usize) {
        let bytes = self.query.as_bytes();
        assert!(length <= bytes.len(), "Prefix length out of bounds");
        self.key.set_bytes(&bytes[..length]);
    }

    /// Returns true if the agent has state.
    pub fn has_state(&self) -> bool {
        self.state.is_some()
    }

    /// Initializes state for complex searches.
    ///
    /// # Errors
    ///
    /// Returns an error if state is already initialized.
    pub fn init_state(&mut self) -> io::Result<()> {
        if self.state.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "State already initialized",
            ));
        }
        self.state = Some(Box::new(State::new()));
        Ok(())
    }

    /// Clears the agent to empty state.
    pub fn clear(&mut self) {
        *self = Agent::new();
    }

    /// Swaps with another agent.
    pub fn swap(&mut self, other: &mut Agent) {
        std::mem::swap(self, other);
    }
}

/// Updates agent's key pointer after copying state.
///
/// In predictive search states, the agent's key points into the state's
/// key buffer. After copying state, we need to update the pointer.
fn update_agent_after_copying_state(state: &State, agent: &mut Agent) {
    match state.status_code() {
        StatusCode::ReadyToPredictiveSearch | StatusCode::EndOfPredictiveSearch => {
            // Key points into state buffer - repoint after copy
            let key_buf = state.key_buf();
            agent.key.set_bytes(key_buf);
        }
        _ => {
            // Key is null or points to query - no update needed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_new() {
        let agent = Agent::new();
        assert_eq!(agent.query().length(), 0);
        assert_eq!(agent.key().length(), 0);
        assert!(!agent.has_state());
    }

    #[test]
    fn test_agent_default() {
        let agent = Agent::default();
        assert_eq!(agent.query().length(), 0);
    }

    #[test]
    fn test_agent_set_query_str() {
        let mut agent = Agent::new();
        agent.set_query_str("hello");

        assert_eq!(agent.query().as_str(), "hello");
        assert_eq!(agent.query().length(), 5);
    }

    #[test]
    fn test_agent_set_query_bytes() {
        let mut agent = Agent::new();
        agent.set_query_bytes(b"world");

        assert_eq!(agent.query().as_bytes(), b"world");
        assert_eq!(agent.query().length(), 5);
    }

    #[test]
    fn test_agent_set_query_id() {
        let mut agent = Agent::new();
        agent.set_query_id(42);

        assert_eq!(agent.query().id(), 42);
    }

    #[test]
    fn test_agent_set_key_str() {
        let mut agent = Agent::new();
        agent.set_key_str("test");

        assert_eq!(agent.key().as_str(), "test");
    }

    #[test]
    fn test_agent_set_key_id() {
        let mut agent = Agent::new();
        agent.set_key_id(100);

        assert_eq!(agent.key().id(), 100);
    }

    #[test]
    fn test_agent_init_state() {
        let mut agent = Agent::new();
        assert!(!agent.has_state());

        agent.init_state().unwrap();
        assert!(agent.has_state());
    }

    #[test]
    fn test_agent_init_state_already_exists() {
        let mut agent = Agent::new();
        agent.init_state().unwrap();

        let result = agent.init_state();
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_state_reset_on_set_query() {
        let mut agent = Agent::new();
        agent.init_state().unwrap();

        {
            let state = agent.state_mut().unwrap();
            state.set_status_code(StatusCode::EndOfPredictiveSearch);
        }

        // Setting query should reset state status code
        agent.set_query_str("new query");

        let state = agent.state().unwrap();
        assert_eq!(state.status_code(), StatusCode::ReadyToAll);
    }

    #[test]
    fn test_agent_clear() {
        let mut agent = Agent::new();
        agent.set_query_str("test");
        agent.set_key_id(10);
        agent.init_state().unwrap();

        agent.clear();

        assert_eq!(agent.query().length(), 0);
        assert_eq!(agent.key().length(), 0);
        assert!(!agent.has_state());
    }

    #[test]
    fn test_agent_swap() {
        let s1 = "query1";
        let s2 = "query2";

        let mut a1 = Agent::new();
        a1.set_query_str(s1);
        a1.set_key_id(1);

        let mut a2 = Agent::new();
        a2.set_query_str(s2);
        a2.set_key_id(2);
        a2.init_state().unwrap();

        a1.swap(&mut a2);

        assert_eq!(a1.query().as_str(), "query2");
        assert_eq!(a1.key().id(), 2);
        assert!(a1.has_state());

        assert_eq!(a2.query().as_str(), "query1");
        assert_eq!(a2.key().id(), 1);
        assert!(!a2.has_state());
    }

    #[test]
    fn test_agent_clone() {
        let mut agent = Agent::new();
        agent.set_query_str("original");
        agent.set_key_id(42);
        agent.init_state().unwrap();

        let cloned = agent.clone();

        assert_eq!(cloned.query().as_str(), "original");
        assert_eq!(cloned.key().id(), 42);
        assert!(cloned.has_state());
    }

    #[test]
    fn test_agent_query_mut() {
        let mut agent = Agent::new();
        agent.query_mut().set_str("mutable");

        assert_eq!(agent.query().as_str(), "mutable");
    }

    #[test]
    fn test_agent_key_mut() {
        let mut agent = Agent::new();
        agent.key_mut().set_id(99);

        assert_eq!(agent.key().id(), 99);
    }
}
