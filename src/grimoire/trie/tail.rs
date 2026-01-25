//! Tail storage for trie suffixes.
//!
//! Ported from:
//! - lib/marisa/grimoire/trie/tail.h
//! - lib/marisa/grimoire/trie/tail.cc
//!
//! Tail stores the suffix parts of trie keys efficiently by merging
//! common suffixes. It supports two modes: text (NULL-terminated) and
//! binary (bit-vector terminated).

use crate::base::TailMode;
use crate::grimoire::vector::bit_vector::BitVector;
use crate::grimoire::vector::vector::Vector;
use std::io;

#[allow(unused_imports)]
use crate::grimoire::io::{Reader, Writer};

/// Tail structure for storing trie suffixes.
///
/// Tail efficiently stores the suffix portions of trie keys by merging
/// common suffixes. It operates in two modes:
/// - Text mode: NULL-terminated strings (space-efficient for text)
/// - Binary mode: bit-vector terminated (supports binary data with NULLs)
pub struct Tail {
    /// Buffer storing the suffix characters.
    buf: Vector<u8>,
    /// Bit vector marking end positions (binary mode only).
    end_flags: BitVector,
}

impl Default for Tail {
    fn default() -> Self {
        Self::new()
    }
}

impl Tail {
    /// Creates a new empty tail.
    pub fn new() -> Self {
        Tail {
            buf: Vector::new(),
            end_flags: BitVector::new(),
        }
    }

    /// Returns the character at the given offset.
    ///
    /// # Arguments
    ///
    /// * `offset` - Offset into the tail buffer
    ///
    /// # Panics
    ///
    /// Panics if offset >= size()
    #[inline]
    pub fn get(&self, offset: usize) -> u8 {
        assert!(offset < self.buf.size(), "Offset out of bounds");
        self.buf[offset]
    }

    /// Returns the tail mode.
    #[inline]
    pub fn mode(&self) -> TailMode {
        if self.end_flags.empty() {
            TailMode::TextTail
        } else {
            TailMode::BinaryTail
        }
    }

    /// Checks if the tail is empty.
    #[inline]
    pub fn empty(&self) -> bool {
        self.buf.empty()
    }

    /// Returns the size of the tail buffer.
    #[inline]
    pub fn size(&self) -> usize {
        self.buf.size()
    }

    /// Returns the total memory size.
    pub fn total_size(&self) -> usize {
        self.buf.total_size() + self.end_flags.total_size()
    }

    /// Returns the I/O size for serialization.
    pub fn io_size(&self) -> usize {
        self.buf.io_size() + self.end_flags.io_size()
    }

    /// Reads tail from a reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - Reader to read from
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails.
    #[allow(dead_code)]
    pub fn read(&mut self, _reader: &mut Reader) -> io::Result<()> {
        // TODO: implement proper I/O
        Ok(())
    }

    /// Writes tail to a writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - Writer to write to
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    #[allow(dead_code)]
    pub fn write(&self, _writer: &mut Writer) -> io::Result<()> {
        // TODO: implement proper I/O
        Ok(())
    }

    /// Restores a key from the tail at the given offset.
    ///
    /// Appends the tail string to the agent's key buffer.
    ///
    /// # Arguments
    ///
    /// * `agent` - Agent containing the state with key buffer
    /// * `offset` - Offset into the tail buffer
    pub fn restore(&self, agent: &mut crate::agent::Agent, offset: usize) {
        assert!(!self.buf.empty(), "Tail buffer is empty");

        let state = agent.state_mut().expect("Agent must have state");

        if self.end_flags.empty() {
            // Text mode: read until NULL terminator
            let mut i = offset;
            while i < self.buf.size() && self.buf[i] != 0 {
                state.key_buf_mut().push(self.buf[i]);
                i += 1;
            }
        } else {
            // Binary mode: read until end flag
            let mut i = offset;
            loop {
                state.key_buf_mut().push(self.buf[i]);
                if self.end_flags.get(i) {
                    break;
                }
                i += 1;
            }
        }
    }

    /// Matches query against tail at the given offset.
    ///
    /// Returns true if the remaining query matches the tail string.
    ///
    /// # Arguments
    ///
    /// * `agent` - Agent containing the query and state
    /// * `offset` - Offset into the tail buffer
    pub fn match_tail(&self, agent: &mut crate::agent::Agent, offset: usize) -> bool {
        assert!(!self.buf.empty(), "Tail buffer is empty");

        // Get query bytes to avoid borrow conflicts
        let query_bytes = agent.query().as_bytes().to_vec();
        let mut query_pos = agent
            .state()
            .expect("Agent must have state")
            .query_pos();

        assert!(query_pos < query_bytes.len(), "Query position out of bounds");

        if self.end_flags.empty() {
            // Text mode
            let start_offset = offset - query_pos;
            loop {
                if self.buf[start_offset + query_pos] != query_bytes[query_pos] {
                    return false;
                }
                query_pos += 1;
                agent.state_mut().expect("Agent must have state").set_query_pos(query_pos);

                if start_offset + query_pos >= self.buf.size()
                    || self.buf[start_offset + query_pos] == 0
                {
                    return true;
                }

                if query_pos >= query_bytes.len() {
                    return false;
                }
            }
        } else {
            // Binary mode
            let mut i = offset;
            loop {
                if self.buf[i] != query_bytes[query_pos] {
                    return false;
                }
                query_pos += 1;
                agent.state_mut().expect("Agent must have state").set_query_pos(query_pos);

                let is_end = self.end_flags.get(i);
                i += 1;

                if is_end {
                    return true;
                }

                if query_pos >= query_bytes.len() {
                    return false;
                }
            }
        }
    }

    /// Matches query prefix against tail and restores the rest.
    ///
    /// Returns true if the remaining query matches the tail prefix,
    /// and appends the full tail string to the key buffer.
    ///
    /// # Arguments
    ///
    /// * `agent` - Agent containing the query and state
    /// * `offset` - Offset into the tail buffer
    pub fn prefix_match(&self, agent: &mut crate::agent::Agent, offset: usize) -> bool {
        assert!(!self.buf.empty(), "Tail buffer is empty");

        // Get query bytes to avoid borrow conflicts
        let query_bytes = agent.query().as_bytes().to_vec();
        let mut query_pos = agent
            .state()
            .expect("Agent must have state")
            .query_pos();

        if self.end_flags.empty() {
            // Text mode
            let start_offset = offset - query_pos;
            loop {
                if self.buf[start_offset + query_pos] != query_bytes[query_pos] {
                    return false;
                }
                let state = agent.state_mut().expect("Agent must have state");
                state.key_buf_mut().push(self.buf[start_offset + query_pos]);
                query_pos += 1;
                state.set_query_pos(query_pos);

                if start_offset + query_pos >= self.buf.size()
                    || self.buf[start_offset + query_pos] == 0
                {
                    return true;
                }

                if query_pos >= query_bytes.len() {
                    break;
                }
            }

            // Append rest of tail
            let state = agent.state_mut().expect("Agent must have state");
            let mut i = start_offset + query_pos;
            while i < self.buf.size() && self.buf[i] != 0 {
                state.key_buf_mut().push(self.buf[i]);
                i += 1;
            }
            return true;
        } else {
            // Binary mode
            let mut i = offset;
            loop {
                if self.buf[i] != query_bytes[query_pos] {
                    return false;
                }
                let state = agent.state_mut().expect("Agent must have state");
                state.key_buf_mut().push(self.buf[i]);
                query_pos += 1;
                state.set_query_pos(query_pos);

                let is_end = self.end_flags.get(i);
                i += 1;

                if is_end {
                    return true;
                }

                if query_pos >= query_bytes.len() {
                    break;
                }
            }

            // Append rest of tail
            let state = agent.state_mut().expect("Agent must have state");
            loop {
                state.key_buf_mut().push(self.buf[i]);
                if self.end_flags.get(i) {
                    break;
                }
                i += 1;
            }
            return true;
        }
    }

    /// Clears the tail.
    pub fn clear(&mut self) {
        let mut temp = Tail::new();
        self.swap(&mut temp);
    }

    /// Swaps with another tail.
    pub fn swap(&mut self, other: &mut Tail) {
        std::mem::swap(&mut self.buf, &mut other.buf);
        std::mem::swap(&mut self.end_flags, &mut other.end_flags);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tail_new() {
        let tail = Tail::new();
        assert!(tail.empty());
        assert_eq!(tail.size(), 0);
        assert_eq!(tail.mode(), TailMode::TextTail);
    }

    #[test]
    fn test_tail_default() {
        let tail = Tail::default();
        assert!(tail.empty());
        assert_eq!(tail.size(), 0);
    }

    #[test]
    fn test_tail_mode() {
        let tail = Tail::new();
        assert_eq!(tail.mode(), TailMode::TextTail);

        // With end_flags, it should be binary mode
        let mut tail_bin = Tail::new();
        tail_bin.end_flags.push_back(true);
        tail_bin.end_flags.build(false, false);
        assert_eq!(tail_bin.mode(), TailMode::BinaryTail);
    }

    #[test]
    fn test_tail_clear() {
        let mut tail = Tail::new();
        tail.buf.push_back(b'a');
        tail.buf.push_back(b'b');

        assert!(!tail.empty());
        tail.clear();
        assert!(tail.empty());
    }

    #[test]
    fn test_tail_swap() {
        let mut tail1 = Tail::new();
        tail1.buf.push_back(b'a');

        let mut tail2 = Tail::new();
        tail2.buf.push_back(b'b');
        tail2.buf.push_back(b'c');

        assert_eq!(tail1.size(), 1);
        assert_eq!(tail2.size(), 2);

        tail1.swap(&mut tail2);

        assert_eq!(tail1.size(), 2);
        assert_eq!(tail2.size(), 1);
    }

    #[test]
    fn test_tail_get() {
        let mut tail = Tail::new();
        tail.buf.push_back(b'h');
        tail.buf.push_back(b'e');
        tail.buf.push_back(b'l');
        tail.buf.push_back(b'l');
        tail.buf.push_back(b'o');

        assert_eq!(tail.get(0), b'h');
        assert_eq!(tail.get(1), b'e');
        assert_eq!(tail.get(4), b'o');
    }

    #[test]
    #[should_panic(expected = "Offset out of bounds")]
    fn test_tail_get_out_of_bounds() {
        let tail = Tail::new();
        tail.get(0);
    }

    #[test]
    fn test_tail_sizes() {
        let tail = Tail::new();
        assert_eq!(tail.size(), 0);

        let total = tail.total_size();
        let io = tail.io_size();

        // Both should be non-negative
        assert!(total >= 0);
        assert!(io >= 0);
    }
}
