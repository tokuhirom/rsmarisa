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

    /// Builds tail storage from entries.
    ///
    /// # Arguments
    ///
    /// * `entries` - Vector of entries to build from
    /// * `offsets` - Output vector for tail offsets
    /// * `mode` - Tail mode (text or binary)
    pub fn build(
        &mut self,
        entries: &mut Vector<crate::grimoire::trie::entry::Entry<'_>>,
        offsets: &mut Vector<u32>,
        mut mode: TailMode,
    ) {
        // Check if any entry contains NULL bytes - if so, use binary mode
        if mode == TailMode::TextTail {
            for i in 0..entries.size() {
                let bytes = entries[i].as_bytes();
                for &b in bytes {
                    if b == 0 {
                        mode = TailMode::BinaryTail;
                        break;
                    }
                }
                if mode == TailMode::BinaryTail {
                    break;
                }
            }
        }

        let mut temp = Tail::new();
        temp.build_(entries, offsets, mode);
        self.swap(&mut temp);
    }

    /// Internal build implementation.
    fn build_(
        &mut self,
        entries: &mut Vector<crate::grimoire::trie::entry::Entry<'_>>,
        offsets: &mut Vector<u32>,
        mode: TailMode,
    ) {
        use crate::grimoire::trie::entry::Entry;

        // Set IDs for all entries
        for i in 0..entries.size() {
            entries[i].set_id(i);
        }

        // Sort entries using algorithm::sort semantics (ascending order)
        // C++ tail.cc uses algorithm::sort which compares in ascending order:
        //   lhs[i] < rhs[i] => lhs comes first
        //   shorter string comes first if it's a prefix
        let entries_slice = entries.as_mut_slice();
        entries_slice.sort_by(|a, b| {
            for i in 0..a.length() {
                if i == b.length() {
                    // a is longer than b, a comes after
                    return std::cmp::Ordering::Greater;
                }
                let a_byte = a.get(i);
                let b_byte = b.get(i);
                if a_byte != b_byte {
                    // ascending order by byte value
                    return a_byte.cmp(&b_byte);
                }
            }
            // a is shorter or equal length
            a.length().cmp(&b.length())
        });

        let mut temp_offsets: Vector<u32> = Vector::new();
        temp_offsets.resize(entries.size(), 0);

        // Process entries in reverse order to find common suffixes
        let dummy = Entry::new();
        let mut last = dummy;

        for i in (0..entries.size()).rev() {
            let current = entries[i];

            assert!(current.length() > 0, "Entry length must be > 0");

            // Find longest common prefix (remember entries are accessed in reverse)
            let mut match_len = 0;
            while match_len < current.length()
                && match_len < last.length()
                && current.get(match_len) == last.get(match_len)
            {
                match_len += 1;
            }

            if match_len == current.length() && last.length() != 0 {
                // Current is a suffix of last - reuse the tail
                temp_offsets[current.id()] =
                    temp_offsets[last.id()] + (last.length() - match_len) as u32;
            } else {
                // Add new entry to tail buffer
                temp_offsets[current.id()] = self.buf.size() as u32;

                // Add bytes in reverse order
                // Note: Entry::get(j) already accesses in reverse (ptr - j),
                // so we need to iterate forward to get reverse storage
                for j in 0..current.length() {
                    self.buf.push_back(current.get(current.length() - 1 - j));
                }

                // Add terminator
                if mode == TailMode::TextTail {
                    self.buf.push_back(0); // NULL terminator
                } else {
                    // Binary mode: add end flags
                    for _ in 0..(current.length() - 1) {
                        self.end_flags.push_back(false);
                    }
                    self.end_flags.push_back(true);
                }

                assert!(
                    self.buf.size() <= u32::MAX as usize,
                    "Tail buffer too large"
                );
            }

            last = current;
        }

        // Build end_flags if in binary mode
        if mode == TailMode::BinaryTail {
            self.end_flags.build(false, false);
        }

        self.buf.shrink();
        offsets.swap(&mut temp_offsets);
    }

    /// Reads tail from a reader.
    ///
    /// Format:
    /// - buf: Vector<u8> (suffix buffer)
    /// - end_flags: BitVector (end markers for binary mode)
    ///
    /// # Arguments
    ///
    /// * `reader` - Reader to read from
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails.
    pub fn read(&mut self, reader: &mut Reader) -> io::Result<()> {
        self.buf.read(reader)?;
        self.end_flags.read(reader)?;
        Ok(())
    }

    /// Writes tail to a writer.
    ///
    /// Format:
    /// - buf: Vector<u8> (suffix buffer)
    /// - end_flags: BitVector (end markers for binary mode)
    ///
    /// # Arguments
    ///
    /// * `writer` - Writer to write to
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn write(&self, writer: &mut Writer) -> io::Result<()> {
        self.buf.write(writer)?;
        self.end_flags.write(writer)?;
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
        // If tail buffer is empty (not built yet), there's nothing to restore
        if self.buf.empty() {
            return;
        }

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
        // If tail buffer is empty (not built yet), cannot match
        if self.buf.empty() {
            return false;
        }

        // Get query bytes to avoid borrow conflicts
        let query_bytes = agent.query().as_bytes().to_vec();
        let mut query_pos = agent.state().expect("Agent must have state").query_pos();

        assert!(
            query_pos < query_bytes.len(),
            "Query position out of bounds"
        );

        if self.end_flags.empty() {
            // Text mode
            // In C++: const char *const ptr = &buf_[offset] - state.query_pos();
            // Then ptr[query_pos + i] accesses buf_[offset + i]
            // We track the initial query_pos and compute: buf[offset + (current_query_pos - initial_query_pos)]
            let initial_query_pos = query_pos;

            loop {
                // Access buf[offset + (query_pos - initial_query_pos)]
                let buf_index = offset + (query_pos - initial_query_pos);
                if buf_index >= self.buf.size() {
                    return false; // Unexpected end of buffer
                }
                if self.buf[buf_index] != query_bytes[query_pos] {
                    return false; // Mismatch
                }
                query_pos += 1;
                agent
                    .state_mut()
                    .expect("Agent must have state")
                    .set_query_pos(query_pos);

                let buf_index = offset + (query_pos - initial_query_pos);
                if buf_index >= self.buf.size() {
                    return false; // Unexpected end of buffer
                }
                if self.buf[buf_index] == 0 {
                    return true; // Found null terminator
                }

                if query_pos >= query_bytes.len() {
                    return false; // Query exhausted but no null terminator
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
                agent
                    .state_mut()
                    .expect("Agent must have state")
                    .set_query_pos(query_pos);

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
        // If tail buffer is empty (not built yet), cannot match
        if self.buf.empty() {
            return false;
        }

        // Get query bytes to avoid borrow conflicts
        let query_bytes = agent.query().as_bytes().to_vec();
        let mut query_pos = agent.state().expect("Agent must have state").query_pos();

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

    #[test]
    fn test_tail_write_read_text_mode() {
        // Rust-specific: Test Tail serialization in text mode
        use crate::grimoire::io::{Reader, Writer};

        // Create tail with text mode data (NULL-terminated strings)
        let mut tail = Tail::new();
        // Add "apple\0" in reverse
        for &c in b"elppa\0" {
            tail.buf.push_back(c);
        }
        // Add "app\0" in reverse
        for &c in b"ppa\0" {
            tail.buf.push_back(c);
        }

        assert_eq!(tail.mode(), TailMode::TextTail);
        assert!(!tail.empty());

        // Write to buffer
        let mut writer = Writer::from_vec(Vec::new());
        tail.write(&mut writer).unwrap();

        let data = writer.into_inner().unwrap();

        // Read back
        let mut reader = Reader::from_bytes(&data);
        let mut tail2 = Tail::new();
        tail2.read(&mut reader).unwrap();

        // Verify
        assert_eq!(tail2.mode(), TailMode::TextTail);
        assert_eq!(tail2.size(), tail.size());
        for i in 0..tail.size() {
            assert_eq!(tail2.get(i), tail.get(i));
        }
    }

    #[test]
    fn test_tail_write_read_binary_mode() {
        // Rust-specific: Test Tail serialization in binary mode
        use crate::grimoire::io::{Reader, Writer};

        // Create tail with binary mode data (bit-vector terminated)
        let mut tail = Tail::new();
        // Add some bytes
        tail.buf.push_back(b'a');
        tail.buf.push_back(0); // NULL byte in data
        tail.buf.push_back(b'b');
        tail.buf.push_back(b'c');

        // Add end flags (mark last byte as end)
        tail.end_flags.push_back(false);
        tail.end_flags.push_back(false);
        tail.end_flags.push_back(false);
        tail.end_flags.push_back(true);
        tail.end_flags.build(false, false);

        assert_eq!(tail.mode(), TailMode::BinaryTail);
        assert!(!tail.empty());

        // Write to buffer
        let mut writer = Writer::from_vec(Vec::new());
        tail.write(&mut writer).unwrap();

        let data = writer.into_inner().unwrap();

        // Read back
        let mut reader = Reader::from_bytes(&data);
        let mut tail2 = Tail::new();
        tail2.read(&mut reader).unwrap();

        // Verify
        assert_eq!(tail2.mode(), TailMode::BinaryTail);
        assert_eq!(tail2.size(), tail.size());
        for i in 0..tail.size() {
            assert_eq!(tail2.get(i), tail.get(i));
        }
        // Verify end flags
        for i in 0..4 {
            assert_eq!(tail2.end_flags.get(i), tail.end_flags.get(i));
        }
    }

    #[test]
    fn test_tail_write_read_empty() {
        // Rust-specific: Test empty Tail serialization
        use crate::grimoire::io::{Reader, Writer};

        let tail = Tail::new();
        assert!(tail.empty());

        // Write to buffer
        let mut writer = Writer::from_vec(Vec::new());
        tail.write(&mut writer).unwrap();

        let data = writer.into_inner().unwrap();

        // Read back
        let mut reader = Reader::from_bytes(&data);
        let mut tail2 = Tail::new();
        tail2.read(&mut reader).unwrap();

        // Verify
        assert!(tail2.empty());
        assert_eq!(tail2.size(), 0);
        assert_eq!(tail2.mode(), TailMode::TextTail);
    }
}
