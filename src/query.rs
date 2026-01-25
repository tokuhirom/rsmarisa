//! Query type for trie searches.
//!
//! Ported from: include/marisa/query.h

use std::fmt;

/// Query represents a search query with a string and optional ID.
///
/// A query can contain either:
/// - A string to search for (ptr + length)
/// - An ID to reverse lookup
#[derive(Clone)]
pub struct Query {
    /// Pointer to query string data (borrowed).
    ptr: Option<*const u8>,
    /// Length of query string.
    length: usize,
    /// Query ID for reverse lookup.
    id: usize,
}

// Manual Debug implementation since raw pointers don't implement Debug
impl fmt::Debug for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Query")
            .field("ptr", &self.ptr.map(|_| "..."))
            .field("length", &self.length)
            .field("id", &self.id)
            .finish()
    }
}

impl Default for Query {
    fn default() -> Self {
        Self::new()
    }
}

impl Query {
    /// Creates a new empty query.
    pub fn new() -> Self {
        Query {
            ptr: None,
            length: 0,
            id: 0,
        }
    }

    /// Returns the character at the specified index.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    pub fn get(&self, i: usize) -> u8 {
        assert!(i < self.length, "Index out of bounds");
        if let Some(ptr) = self.ptr {
            unsafe { *ptr.add(i) }
        } else {
            panic!("Query has no string data");
        }
    }

    /// Sets the query from a string slice.
    pub fn set_str(&mut self, s: &str) {
        self.ptr = Some(s.as_ptr());
        self.length = s.len();
    }

    /// Sets the query from a byte slice.
    pub fn set_bytes(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            self.ptr = None;
            self.length = 0;
        } else {
            self.ptr = Some(bytes.as_ptr());
            self.length = bytes.len();
        }
    }

    /// Sets the query ID.
    pub fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    /// Returns the query as a byte slice.
    ///
    /// Returns an empty slice if no string is set.
    pub fn as_bytes(&self) -> &[u8] {
        if let Some(ptr) = self.ptr {
            unsafe { std::slice::from_raw_parts(ptr, self.length) }
        } else {
            &[]
        }
    }

    /// Returns the query string as a str reference.
    ///
    /// # Panics
    ///
    /// Panics if the query contains invalid UTF-8.
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(self.as_bytes()).expect("Invalid UTF-8 in query")
    }

    /// Returns a pointer to the query data.
    pub fn ptr(&self) -> Option<*const u8> {
        self.ptr
    }

    /// Returns the length of the query string.
    pub fn length(&self) -> usize {
        self.length
    }

    /// Returns the query ID.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Clears the query to empty state.
    pub fn clear(&mut self) {
        *self = Query::new();
    }

    /// Swaps with another query.
    pub fn swap(&mut self, other: &mut Query) {
        std::mem::swap(self, other);
    }
}

// Safety: Query only holds a pointer that must remain valid for its lifetime.
// The user is responsible for ensuring the borrowed data outlives the Query.
unsafe impl Send for Query {}
unsafe impl Sync for Query {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_new() {
        let query = Query::new();
        assert_eq!(query.length(), 0);
        assert_eq!(query.id(), 0);
        assert_eq!(query.as_bytes(), &[]);
    }

    #[test]
    fn test_query_default() {
        let query = Query::default();
        assert_eq!(query.length(), 0);
    }

    #[test]
    fn test_query_set_str() {
        let s = "hello";
        let mut query = Query::new();
        query.set_str(s);

        assert_eq!(query.length(), 5);
        assert_eq!(query.as_str(), "hello");
        assert_eq!(query.as_bytes(), b"hello");
    }

    #[test]
    fn test_query_set_bytes() {
        let bytes = b"world";
        let mut query = Query::new();
        query.set_bytes(bytes);

        assert_eq!(query.length(), 5);
        assert_eq!(query.as_bytes(), b"world");
    }

    #[test]
    fn test_query_set_empty_bytes() {
        let mut query = Query::new();
        query.set_str("test");
        query.set_bytes(&[]);

        assert_eq!(query.length(), 0);
        assert_eq!(query.as_bytes(), &[]);
    }

    #[test]
    fn test_query_get() {
        let s = "test";
        let mut query = Query::new();
        query.set_str(s);

        assert_eq!(query.get(0), b't');
        assert_eq!(query.get(1), b'e');
        assert_eq!(query.get(2), b's');
        assert_eq!(query.get(3), b't');
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_query_get_out_of_bounds() {
        let s = "test";
        let mut query = Query::new();
        query.set_str(s);
        query.get(4);
    }

    #[test]
    fn test_query_set_id() {
        let mut query = Query::new();
        query.set_id(42);
        assert_eq!(query.id(), 42);
    }

    #[test]
    fn test_query_clear() {
        let mut query = Query::new();
        query.set_str("test");
        query.set_id(10);

        query.clear();

        assert_eq!(query.length(), 0);
        assert_eq!(query.id(), 0);
        assert_eq!(query.as_bytes(), &[]);
    }

    #[test]
    fn test_query_swap() {
        let s1 = "hello";
        let s2 = "world";

        let mut q1 = Query::new();
        q1.set_str(s1);
        q1.set_id(1);

        let mut q2 = Query::new();
        q2.set_str(s2);
        q2.set_id(2);

        q1.swap(&mut q2);

        assert_eq!(q1.as_str(), "world");
        assert_eq!(q1.id(), 2);
        assert_eq!(q2.as_str(), "hello");
        assert_eq!(q2.id(), 1);
    }

    #[test]
    fn test_query_with_unicode() {
        let s = "こんにちは";
        let mut query = Query::new();
        query.set_str(s);

        assert_eq!(query.length(), s.len());
        assert_eq!(query.as_str(), "こんにちは");
    }
}
