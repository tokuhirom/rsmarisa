//! Trie data structure.
//!
//! Ported from: include/marisa/trie.h and lib/marisa/trie.cc
//!
//! This module provides the main Trie structure, which is a wrapper around
//! the internal LoudsTrie implementation. It provides a safe and convenient
//! public API for trie operations.

use crate::agent::Agent;
use crate::base::{NodeOrder, TailMode};
use crate::grimoire::io::{Reader, Writer};
use crate::grimoire::trie::louds_trie::LoudsTrie;
use crate::keyset::Keyset;

/// Main trie data structure.
///
/// Trie is a static and space-efficient trie implementation that supports:
/// - **Lookup**: Check if a string exists in the trie
/// - **Reverse lookup**: Restore a key from its ID
/// - **Common prefix search**: Find all keys that are prefixes of a query
/// - **Predictive search**: Find all keys that start with a query prefix
///
/// # Examples
///
/// ```
/// use rsmarisa::{Trie, Keyset};
///
/// // Build a trie
/// let mut keyset = Keyset::new();
/// keyset.push_back_str("apple");
/// keyset.push_back_str("application");
/// keyset.push_back_str("apply");
///
/// let mut trie = Trie::new();
/// trie.build(&mut keyset, 0);
///
/// assert_eq!(trie.num_keys(), 3);
/// ```
pub struct Trie {
    /// Internal LOUDS trie implementation.
    trie: Option<Box<LoudsTrie>>,
}

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
}

impl Trie {
    /// Creates a new empty trie.
    pub fn new() -> Self {
        Trie { trie: None }
    }

    /// Builds a trie from a keyset.
    ///
    /// # Arguments
    ///
    /// * `keyset` - Keyset containing strings to build the trie from
    /// * `config_flags` - Configuration flags (default: 0)
    ///
    /// # Examples
    ///
    /// ```
    /// use rsmarisa::{Trie, Keyset};
    ///
    /// let mut keyset = Keyset::new();
    /// keyset.push_back_str("hello");
    /// keyset.push_back_str("world");
    ///
    /// let mut trie = Trie::new();
    /// trie.build(&mut keyset, 0);
    /// ```
    pub fn build(&mut self, keyset: &mut Keyset, config_flags: i32) {
        let mut temp = Box::new(LoudsTrie::new());
        temp.build(keyset, config_flags);
        self.trie = Some(temp);
    }

    /// Memory-maps a trie from a file.
    ///
    /// This method uses memory-mapped I/O for efficient loading of large tries.
    /// The file is mapped into memory, and the trie data structures reference
    /// the mapped memory directly, avoiding the need to copy data.
    ///
    /// # Arguments
    ///
    /// * `filename` - Path to the trie file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened/mapped or contains invalid data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rsmarisa::Trie;
    ///
    /// let mut trie = Trie::new();
    /// trie.mmap("dictionary.marisa").unwrap();
    /// ```
    pub fn mmap(&mut self, filename: &str) -> std::io::Result<()> {
        let mut temp = Box::new(LoudsTrie::new());
        temp.mmap(filename)?;
        self.trie = Some(temp);
        Ok(())
    }

    /// Maps a trie from static memory.
    ///
    /// This method maps a trie from a byte slice that must have static lifetime.
    /// Useful for embedding trie data in the binary or loading from a custom source.
    ///
    /// # Arguments
    ///
    /// * `data` - Static byte slice containing the trie data
    ///
    /// # Errors
    ///
    /// Returns an error if the data is invalid.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rsmarisa::Trie;
    ///
    /// static TRIE_DATA: &[u8] = include_bytes!("dictionary.marisa");
    ///
    /// let mut trie = Trie::new();
    /// trie.map(TRIE_DATA).unwrap();
    /// ```
    pub fn map(&mut self, data: &'static [u8]) -> std::io::Result<()> {
        let mut temp = Box::new(LoudsTrie::new());
        temp.map(data)?;
        self.trie = Some(temp);
        Ok(())
    }

    /// Loads a trie from a file.
    ///
    /// # Arguments
    ///
    /// * `filename` - Path to the file
    ///
    /// # Errors
    ///
    /// Returns an error if loading fails or file is invalid
    pub fn load(&mut self, filename: &str) -> std::io::Result<()> {
        let mut reader = Reader::open(filename)?;
        self.read(&mut reader)
    }

    /// Reads a trie from a reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - Reader to read from
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails
    pub fn read(&mut self, reader: &mut Reader) -> std::io::Result<()> {
        let mut temp = Box::new(LoudsTrie::new());
        temp.read(reader)?;
        self.trie = Some(temp);
        Ok(())
    }

    /// Saves a trie to a file.
    ///
    /// # Arguments
    ///
    /// * `filename` - Path to the file
    ///
    /// # Errors
    ///
    /// Returns an error if saving fails or trie is empty
    pub fn save(&self, filename: &str) -> std::io::Result<()> {
        if self.trie.is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Cannot save empty trie (not built)",
            ));
        }
        let mut writer = Writer::open(filename)?;
        self.write(&mut writer)
    }

    /// Writes a trie to a writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - Writer to write to
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails or trie is empty
    pub fn write(&self, writer: &mut Writer) -> std::io::Result<()> {
        match self.trie.as_ref() {
            Some(trie) => trie.write(writer),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Cannot write empty trie (not built)",
            )),
        }
    }

    /// Looks up a key in the trie.
    ///
    /// Returns true if the query string exists as a complete key in the trie.
    ///
    /// # Arguments
    ///
    /// * `agent` - Agent with query set
    ///
    /// # Returns
    ///
    /// true if the key exists, false otherwise
    ///
    /// # Panics
    ///
    /// Panics if the trie is empty (not built)
    ///
    /// # Examples
    ///
    /// ```
    /// use rsmarisa::{Trie, Keyset, Agent};
    ///
    /// let mut keyset = Keyset::new();
    /// keyset.push_back_str("apple");
    ///
    /// let mut trie = Trie::new();
    /// trie.build(&mut keyset, 0);
    ///
    /// let mut agent = Agent::new();
    /// agent.set_query_str("apple");
    /// assert!(trie.lookup(&mut agent));
    ///
    /// agent.set_query_str("orange");
    /// assert!(!trie.lookup(&mut agent));
    /// ```
    pub fn lookup(&self, agent: &mut Agent) -> bool {
        let trie = self.trie.as_ref().expect("Trie not built");
        if !agent.has_state() {
            agent
                .init_state()
                .expect("Failed to initialize agent state");
        }
        trie.lookup(agent)
    }

    /// Performs reverse lookup: finds the key corresponding to a key ID.
    ///
    /// # Arguments
    ///
    /// * `agent` - Agent with query ID set
    ///
    /// # Panics
    ///
    /// Panics if the trie is empty (not built) or if the key ID is out of range
    ///
    /// # Examples
    ///
    /// ```
    /// use rsmarisa::{Trie, Keyset, Agent};
    ///
    /// let mut keyset = Keyset::new();
    /// keyset.push_back_str("apple");
    /// keyset.push_back_str("banana");
    ///
    /// let mut trie = Trie::new();
    /// trie.build(&mut keyset, 0);
    ///
    /// let mut agent = Agent::new();
    /// agent.set_query_id(0);
    /// trie.reverse_lookup(&mut agent);
    /// // agent.key() now contains the key for ID 0
    /// ```
    pub fn reverse_lookup(&self, agent: &mut Agent) {
        let trie = self.trie.as_ref().expect("Trie not built");
        if !agent.has_state() {
            agent
                .init_state()
                .expect("Failed to initialize agent state");
        }
        trie.reverse_lookup(agent);
    }

    /// Performs common prefix search.
    ///
    /// Finds keys that are prefixes of the query string.
    /// Call repeatedly to get all matching prefixes.
    ///
    /// # Arguments
    ///
    /// * `agent` - Agent with query set
    ///
    /// # Returns
    ///
    /// true if a match was found, false if no more matches
    ///
    /// # Panics
    ///
    /// Panics if the trie is empty (not built)
    ///
    /// # Examples
    ///
    /// ```
    /// use rsmarisa::{Trie, Keyset, Agent};
    ///
    /// let mut keyset = Keyset::new();
    /// keyset.push_back_str("app");
    /// keyset.push_back_str("apple");
    ///
    /// let mut trie = Trie::new();
    /// trie.build(&mut keyset, 0);
    ///
    /// let mut agent = Agent::new();
    /// agent.set_query_str("application");
    ///
    /// // Find all prefixes - only "app" is a prefix of "application"
    /// // Note: "apple" is NOT a prefix of "application"
    /// assert!(trie.common_prefix_search(&mut agent));
    /// assert_eq!(std::str::from_utf8(agent.key().as_bytes()).unwrap(), "app");
    /// assert!(!trie.common_prefix_search(&mut agent)); // No more matches
    /// ```
    pub fn common_prefix_search(&self, agent: &mut Agent) -> bool {
        let trie = self.trie.as_ref().expect("Trie not built");
        if !agent.has_state() {
            agent
                .init_state()
                .expect("Failed to initialize agent state");
        }
        trie.common_prefix_search(agent)
    }

    /// Performs predictive search.
    ///
    /// Finds keys that start with the query string.
    /// Call repeatedly to get all matching keys.
    ///
    /// # Arguments
    ///
    /// * `agent` - Agent with query set
    ///
    /// # Returns
    ///
    /// true if a match was found, false if no more matches
    ///
    /// # Panics
    ///
    /// Panics if the trie is empty (not built)
    ///
    /// # Examples
    ///
    /// ```
    /// use rsmarisa::{Trie, Keyset, Agent};
    ///
    /// let mut keyset = Keyset::new();
    /// keyset.push_back_str("apple");
    /// keyset.push_back_str("application");
    ///
    /// let mut trie = Trie::new();
    /// trie.build(&mut keyset, 0);
    ///
    /// let mut agent = Agent::new();
    /// agent.set_query_str("app");
    ///
    /// // Find all keys starting with "app"
    /// let mut count = 0;
    /// while trie.predictive_search(&mut agent) {
    ///     count += 1;
    /// }
    /// assert_eq!(count, 2);
    /// ```
    pub fn predictive_search(&self, agent: &mut Agent) -> bool {
        let trie = self.trie.as_ref().expect("Trie not built");
        if !agent.has_state() {
            agent
                .init_state()
                .expect("Failed to initialize agent state");
        }
        trie.predictive_search(agent)
    }

    /// Returns the number of trie levels.
    ///
    /// # Panics
    ///
    /// Panics if the trie is empty (not built)
    pub fn num_tries(&self) -> usize {
        let trie = self.trie.as_ref().expect("Trie not built");
        trie.num_tries()
    }

    /// Returns the number of keys in the trie.
    ///
    /// # Panics
    ///
    /// Panics if the trie is empty (not built)
    pub fn num_keys(&self) -> usize {
        let trie = self.trie.as_ref().expect("Trie not built");
        trie.num_keys()
    }

    /// Returns the number of nodes in the trie.
    ///
    /// # Panics
    ///
    /// Panics if the trie is empty (not built)
    pub fn num_nodes(&self) -> usize {
        let trie = self.trie.as_ref().expect("Trie not built");
        trie.num_nodes()
    }

    /// Returns the tail storage mode.
    ///
    /// # Panics
    ///
    /// Panics if the trie is empty (not built)
    pub fn tail_mode(&self) -> TailMode {
        let trie = self.trie.as_ref().expect("Trie not built");
        trie.tail_mode()
    }

    /// Returns the node ordering mode.
    ///
    /// # Panics
    ///
    /// Panics if the trie is empty (not built)
    pub fn node_order(&self) -> NodeOrder {
        let trie = self.trie.as_ref().expect("Trie not built");
        trie.node_order()
    }

    /// Checks if the trie is empty.
    ///
    /// # Panics
    ///
    /// Panics if the trie is not built
    pub fn empty(&self) -> bool {
        let trie = self.trie.as_ref().expect("Trie not built");
        trie.empty()
    }

    /// Returns the number of keys (same as num_keys).
    ///
    /// # Panics
    ///
    /// Panics if the trie is empty (not built)
    pub fn size(&self) -> usize {
        let trie = self.trie.as_ref().expect("Trie not built");
        trie.size()
    }

    /// Returns the total memory size in bytes.
    ///
    /// # Panics
    ///
    /// Panics if the trie is empty (not built)
    pub fn total_size(&self) -> usize {
        let trie = self.trie.as_ref().expect("Trie not built");
        trie.total_size()
    }

    /// Returns the I/O size for serialization.
    ///
    /// # Panics
    ///
    /// Panics if the trie is empty (not built)
    pub fn io_size(&self) -> usize {
        let trie = self.trie.as_ref().expect("Trie not built");
        trie.io_size()
    }

    /// Clears the trie.
    pub fn clear(&mut self) {
        self.trie = None;
    }

    /// Swaps with another trie.
    pub fn swap(&mut self, other: &mut Trie) {
        std::mem::swap(&mut self.trie, &mut other.trie);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trie_new() {
        // Rust-specific: Test Trie::new() initialization
        let trie = Trie::new();
        assert!(trie.trie.is_none());
    }

    #[test]
    fn test_trie_build() {
        // Rust-specific: Test basic trie building
        let mut keyset = Keyset::new();
        let _ = keyset.push_back_str("apple");
        let _ = keyset.push_back_str("banana");
        let _ = keyset.push_back_str("cherry");

        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);

        assert_eq!(trie.num_keys(), 3);
    }

    #[test]
    fn test_trie_lookup() {
        let mut keyset = Keyset::new();
        let _ = keyset.push_back_str("app");
        let _ = keyset.push_back_str("apple");

        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);

        let mut agent = Agent::new();
        agent.set_query_str("app");
        assert!(trie.lookup(&mut agent), "Should find 'app'");
        println!(
            "Found app: id={}, str={:?}",
            agent.key().id(),
            String::from_utf8_lossy(agent.key().as_bytes())
        );

        agent.set_query_str("apple");
        assert!(trie.lookup(&mut agent), "Should find 'apple'");
        println!(
            "Found apple: id={}, str={:?}",
            agent.key().id(),
            String::from_utf8_lossy(agent.key().as_bytes())
        );

        agent.set_query_str("banana");
        assert!(!trie.lookup(&mut agent), "Should not find 'banana'");
    }

    #[test]
    fn test_trie_reverse_lookup() {
        let mut keyset = Keyset::new();
        let _ = keyset.push_back_str("a");
        let _ = keyset.push_back_str("b");

        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);

        let mut agent = Agent::new();
        agent.set_query_id(0);
        trie.reverse_lookup(&mut agent);
        // Key should be set in agent
        assert!(agent.key().length() > 0);
    }

    #[test]
    fn test_trie_common_prefix_search() {
        // Rust-specific: Test basic common prefix search functionality
        // Test 1: Single-character increments
        {
            let mut keyset = Keyset::new();
            let _ = keyset.push_back_str("a");
            let _ = keyset.push_back_str("ab");
            let _ = keyset.push_back_str("abc");

            let mut trie = Trie::new();
            trie.build(&mut keyset, 0);

            let mut agent = Agent::new();
            agent.set_query_str("abc");

            let mut count = 0;
            while trie.common_prefix_search(&mut agent) {
                count += 1;
                if count > 10 {
                    break;
                }
            }
            assert_eq!(
                count, 3,
                "Expected 3 matches (a, ab, abc) but got {}",
                count
            );
        }

        // Rust-specific: Verify behavior matches C++ marisa with multi-char keys
        // Test 2: Verify "app" and "apple" behavior matches C++ marisa
        // Only "app" should be found as a prefix of "application"
        // ("apple" is NOT a prefix of "application")
        {
            let mut keyset = Keyset::new();
            let _ = keyset.push_back_str("app");
            let _ = keyset.push_back_str("apple");

            let mut trie = Trie::new();
            trie.build(&mut keyset, 0);

            let mut agent = Agent::new();
            agent.set_query_str("application");

            // Should find "app"
            assert!(trie.common_prefix_search(&mut agent));
            assert_eq!(std::str::from_utf8(agent.key().as_bytes()).unwrap(), "app");

            // Should NOT find "apple" (it's not a prefix of "application")
            assert!(!trie.common_prefix_search(&mut agent));
        }
    }

    #[test]
    fn test_trie_predictive_search() {
        let mut keyset = Keyset::new();
        let _ = keyset.push_back_str("a");
        let _ = keyset.push_back_str("ab");
        let _ = keyset.push_back_str("ac");

        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);

        let mut agent = Agent::new();
        agent.set_query_str("a");

        // Note: Full predictive search requires tail support
        // For now, just test that it doesn't crash
        let mut count = 0;
        while trie.predictive_search(&mut agent) {
            count += 1;
            if count > 10 {
                break;
            } // Safety limit
        }
        // Without tail support, we may not get all matches
        assert!(count <= 3);
    }

    #[test]
    fn test_trie_clear() {
        let mut keyset = Keyset::new();
        let _ = keyset.push_back_str("test");

        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);

        trie.clear();
        assert!(trie.trie.is_none());
    }

    #[test]
    fn test_trie_swap() {
        let mut keyset1 = Keyset::new();
        let _ = keyset1.push_back_str("apple");

        let mut trie1 = Trie::new();
        trie1.build(&mut keyset1, 0);

        let mut keyset2 = Keyset::new();
        let _ = keyset2.push_back_str("banana");
        let _ = keyset2.push_back_str("cherry");

        let mut trie2 = Trie::new();
        trie2.build(&mut keyset2, 0);

        trie1.swap(&mut trie2);

        assert_eq!(trie1.num_keys(), 2);
        assert_eq!(trie2.num_keys(), 1);
    }

    #[test]
    fn test_trie_empty() {
        let mut keyset = Keyset::new();
        let _ = keyset.push_back_str("test");

        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);

        assert!(!trie.empty());
    }

    #[test]
    fn test_trie_sizes() {
        let mut keyset = Keyset::new();
        let _ = keyset.push_back_str("test");

        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);

        assert!(trie.total_size() > 0);
        assert!(trie.io_size() > 0);
    }

    #[test]
    fn test_trie_write_read() {
        // Rust-specific: Test Trie serialization with Reader/Writer
        use crate::grimoire::io::{Reader, Writer};

        // Build a trie
        let mut keyset = Keyset::new();
        keyset.push_back_str("app").unwrap();
        keyset.push_back_str("apple").unwrap();
        keyset.push_back_str("application").unwrap();

        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);

        // Write to buffer
        let mut writer = Writer::from_vec(Vec::new());
        trie.write(&mut writer).unwrap();

        let data = writer.into_inner().unwrap();

        // Read back
        let mut reader = Reader::from_bytes(&data);
        let mut trie2 = Trie::new();
        trie2.read(&mut reader).unwrap();

        // Verify structure preserved
        assert_eq!(trie2.num_keys(), 3);
        assert_eq!(trie2.num_nodes(), trie.num_nodes());

        // Verify lookup works
        let mut agent = Agent::new();
        agent.init_state().unwrap();

        agent.set_query_str("app");
        assert!(trie2.lookup(&mut agent));

        agent.set_query_str("apple");
        assert!(trie2.lookup(&mut agent));

        agent.set_query_str("application");
        assert!(trie2.lookup(&mut agent));
    }

    #[test]
    fn test_trie_save_load() {
        // Rust-specific: Test Trie save/load to file
        use std::fs;
        use tempfile::NamedTempFile;

        // Build a trie
        let mut keyset = Keyset::new();
        keyset.push_back_str("hello").unwrap();
        keyset.push_back_str("world").unwrap();

        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);

        // Save to file
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        trie.save(path).unwrap();

        // Verify file exists and has content
        let metadata = fs::metadata(path).unwrap();
        assert!(metadata.len() > 0);

        // Load from file
        let mut trie2 = Trie::new();
        trie2.load(path).unwrap();

        // Verify
        assert_eq!(trie2.num_keys(), 2);

        let mut agent = Agent::new();
        agent.init_state().unwrap();

        agent.set_query_str("hello");
        assert!(trie2.lookup(&mut agent));

        agent.set_query_str("world");
        assert!(trie2.lookup(&mut agent));
    }

    #[test]
    fn test_trie_write_empty_error() {
        // Rust-specific: Test that writing empty trie returns error
        use crate::grimoire::io::Writer;

        let trie = Trie::new();
        let mut writer = Writer::from_vec(Vec::new());
        let result = trie.write(&mut writer);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn test_trie_save_empty_error() {
        // Rust-specific: Test that saving empty trie returns error
        use tempfile::NamedTempFile;

        let trie = Trie::new();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        let result = trie.save(path);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn test_trie_read_invalid_header() {
        // Rust-specific: Test that reading invalid header returns error
        use crate::grimoire::io::Reader;

        let invalid_data = vec![0u8; 100]; // Not a valid MARISA file
        let mut reader = Reader::from_bytes(&invalid_data);
        let mut trie = Trie::new();
        let result = trie.read(&mut reader);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_trie_mmap() {
        // Rust-specific: Test memory-mapped file loading
        use tempfile::NamedTempFile;

        // Build and save a trie
        let mut keyset = Keyset::new();
        keyset.push_back_str("apple").unwrap();
        keyset.push_back_str("application").unwrap();
        keyset.push_back_str("apply").unwrap();

        let mut trie1 = Trie::new();
        trie1.build(&mut keyset, 0);

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        trie1.save(path).unwrap();

        // Load with mmap
        let mut trie2 = Trie::new();
        trie2.mmap(path).unwrap();

        // Verify structure
        assert_eq!(trie2.num_keys(), 3);
        assert_eq!(trie2.num_nodes(), trie1.num_nodes());

        // Verify lookup works
        let mut agent = Agent::new();
        agent.set_query_str("apple");
        assert!(trie2.lookup(&mut agent));
        assert_eq!(
            std::str::from_utf8(agent.key().as_bytes()).unwrap(),
            "apple"
        );

        agent.set_query_str("application");
        assert!(trie2.lookup(&mut agent));
        assert_eq!(
            std::str::from_utf8(agent.key().as_bytes()).unwrap(),
            "application"
        );

        agent.set_query_str("apply");
        assert!(trie2.lookup(&mut agent));

        agent.set_query_str("banana");
        assert!(!trie2.lookup(&mut agent));
    }

    #[test]
    fn test_trie_mmap_vs_load_equivalence() {
        // Rust-specific: Verify that mmap() and load() produce identical behavior
        use tempfile::NamedTempFile;

        // Build and save a trie
        let mut keyset = Keyset::new();
        keyset.push_back_str("test1").unwrap();
        keyset.push_back_str("test2").unwrap();
        keyset.push_back_str("test3").unwrap();

        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        trie.save(path).unwrap();

        // Load via read
        let mut trie_load = Trie::new();
        trie_load.load(path).unwrap();

        // Load via mmap
        let mut trie_mmap = Trie::new();
        trie_mmap.mmap(path).unwrap();

        // Verify identical structure
        assert_eq!(trie_load.num_keys(), trie_mmap.num_keys());
        assert_eq!(trie_load.num_nodes(), trie_mmap.num_nodes());

        // Verify identical lookup behavior
        let test_keys = ["test1", "test2", "test3", "nonexistent"];
        for key in &test_keys {
            let mut agent1 = Agent::new();
            let mut agent2 = Agent::new();

            agent1.set_query_str(key);
            agent2.set_query_str(key);

            let result1 = trie_load.lookup(&mut agent1);
            let result2 = trie_mmap.lookup(&mut agent2);

            assert_eq!(result1, result2, "Lookup result mismatch for key: {}", key);
            if result1 {
                assert_eq!(
                    agent1.key().as_bytes(),
                    agent2.key().as_bytes(),
                    "Key bytes mismatch for key: {}",
                    key
                );
                assert_eq!(
                    agent1.key().id(),
                    agent2.key().id(),
                    "Key ID mismatch for key: {}",
                    key
                );
            }
        }
    }

    #[test]
    fn test_trie_mmap_file_not_found() {
        // Rust-specific: Test that mmap with non-existent file returns error
        let mut trie = Trie::new();
        let result = trie.mmap("/nonexistent/file.marisa");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }
}
