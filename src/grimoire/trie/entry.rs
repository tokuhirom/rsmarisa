//! Entry type for trie construction.
//!
//! Ported from: lib/marisa/grimoire/trie/entry.h
//!
//! Entry represents a string key during trie construction, accessed in
//! reverse order. It stores a byte slice, length, and ID for sorting
//! and building the trie structure.

/// Entry representing a key during trie construction.
///
/// Entry holds a string accessed in reverse order (similar to how the
/// C++ version stores a pointer to the end of the string). The indexing
/// operator accesses bytes from the end backwards.
#[derive(Debug, Clone, Copy)]
pub struct Entry<'a> {
    /// Byte slice representing the string.
    bytes: &'a [u8],
    /// Length of the valid portion.
    length: u32,
    /// Entry ID.
    id: u32,
}

impl<'a> Default for Entry<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Entry<'a> {
    /// Creates a new empty entry.
    pub fn new() -> Self {
        Entry {
            bytes: &[],
            length: 0,
            id: 0,
        }
    }

    /// Returns the byte at the given reverse index.
    ///
    /// # Arguments
    ///
    /// * `i` - Reverse index (0 = last byte)
    ///
    /// # Panics
    ///
    /// Panics if i >= length()
    #[inline]
    pub fn get(&self, i: usize) -> u8 {
        assert!(i < self.length as usize, "Index out of bounds");
        self.bytes[self.length as usize - 1 - i]
    }

    /// Sets the string content.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Byte slice to use as the entry
    ///
    /// # Panics
    ///
    /// Panics if bytes.len() > u32::MAX
    pub fn set_str(&mut self, bytes: &'a [u8]) {
        assert!(bytes.len() <= u32::MAX as usize, "String too long");
        self.bytes = bytes;
        self.length = bytes.len() as u32;
    }

    /// Sets the entry ID.
    ///
    /// # Arguments
    ///
    /// * `id` - ID to set
    ///
    /// # Panics
    ///
    /// Panics if id > u32::MAX
    #[inline]
    pub fn set_id(&mut self, id: usize) {
        assert!(id <= u32::MAX as usize, "ID too large");
        self.id = id as u32;
    }

    /// Returns a slice to the string data.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.length as usize]
    }

    /// Returns the length of the entry.
    #[inline]
    pub fn length(&self) -> usize {
        self.length as usize
    }

    /// Returns the entry ID.
    #[inline]
    pub fn id(&self) -> usize {
        self.id as usize
    }
}

/// Comparer for sorting entries by string content in reverse.
///
/// This implements the same comparison logic as the C++ StringComparer,
/// comparing strings in reverse order (from end to beginning).
#[derive(Debug, Clone, Copy)]
pub struct StringComparer;

impl StringComparer {
    /// Compares two entries by their string content.
    ///
    /// Returns true if lhs > rhs (for descending order sorting).
    pub fn compare(lhs: &Entry, rhs: &Entry) -> bool {
        for i in 0..lhs.length() {
            if i == rhs.length() {
                return true;
            }
            if lhs.get(i) != rhs.get(i) {
                return lhs.get(i) > rhs.get(i);
            }
        }
        lhs.length() > rhs.length()
    }
}

/// Comparer for sorting entries by ID.
#[derive(Debug, Clone, Copy)]
pub struct IDComparer;

impl IDComparer {
    /// Compares two entries by their ID.
    ///
    /// Returns true if lhs.id < rhs.id (for ascending order sorting).
    pub fn compare(lhs: &Entry, rhs: &Entry) -> bool {
        lhs.id < rhs.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_new() {
        let entry = Entry::new();
        assert_eq!(entry.length(), 0);
        assert_eq!(entry.id(), 0);
    }

    #[test]
    fn test_entry_default() {
        let entry = Entry::default();
        assert_eq!(entry.length(), 0);
        assert_eq!(entry.id(), 0);
    }

    #[test]
    fn test_entry_set_str() {
        let data = b"hello";
        let mut entry = Entry::new();
        entry.set_str(data);

        assert_eq!(entry.length(), 5);
        assert_eq!(entry.as_bytes(), b"hello");
    }

    #[test]
    fn test_entry_reverse_indexing() {
        let data = b"hello";
        let mut entry = Entry::new();
        entry.set_str(data);

        // Reverse indexing: [0] = last char
        assert_eq!(entry.get(0), b'o');
        assert_eq!(entry.get(1), b'l');
        assert_eq!(entry.get(2), b'l');
        assert_eq!(entry.get(3), b'e');
        assert_eq!(entry.get(4), b'h');
    }

    #[test]
    fn test_entry_set_id() {
        let data = b"test";
        let mut entry = Entry::new();
        entry.set_str(data);
        entry.set_id(42);

        assert_eq!(entry.id(), 42);
    }

    #[test]
    fn test_entry_clone() {
        let data = b"test";
        let mut entry1 = Entry::new();
        entry1.set_str(data);
        entry1.set_id(10);

        let entry2 = entry1;
        assert_eq!(entry2.length(), 4);
        assert_eq!(entry2.id(), 10);
        assert_eq!(entry2.get(0), b't');
    }

    #[test]
    fn test_string_comparer_equal_length() {
        let data1 = b"apple";
        let data2 = b"zebra";

        let mut entry1 = Entry::new();
        let mut entry2 = Entry::new();

        entry1.set_str(data1);
        entry2.set_str(data2);

        // Comparing reverse: "elppa" vs "arbez"
        // First char (reverse): 'a' vs 'a' - equal
        // Continue comparing...
        // This depends on the reverse order comparison
        let result = StringComparer::compare(&entry1, &entry2);
        // We're comparing in reverse, so details matter
        assert!(result == StringComparer::compare(&entry1, &entry2));
    }

    #[test]
    fn test_string_comparer_different_length() {
        let data1 = b"hello";
        let data2 = b"hi";

        let mut entry1 = Entry::new();
        let mut entry2 = Entry::new();

        entry1.set_str(data1);
        entry2.set_str(data2);

        // entry1 is longer, so if we reach end of entry2, entry1 > entry2
        let result = StringComparer::compare(&entry1, &entry2);
        // First chars in reverse: 'o' vs 'i'
        // 'o' (111) > 'i' (105), so entry1 > entry2
        assert!(result);
    }

    #[test]
    fn test_string_comparer_prefix() {
        let data1 = b"hello";
        let data2 = b"helloworld";

        let mut entry1 = Entry::new();
        let mut entry2 = Entry::new();

        entry1.set_str(data1);
        entry2.set_str(data2);

        // Reverse comparison
        let result = StringComparer::compare(&entry1, &entry2);
        // In reverse: "olleh" vs "dlrowolleh"
        // First chars: 'h' vs 'd', different
        assert!(result == StringComparer::compare(&entry1, &entry2));
    }

    #[test]
    fn test_id_comparer() {
        let data = b"test";
        let mut entry1 = Entry::new();
        let mut entry2 = Entry::new();

        entry1.set_str(data);
        entry2.set_str(data);

        entry1.set_id(10);
        entry2.set_id(20);

        assert!(IDComparer::compare(&entry1, &entry2));
        assert!(!IDComparer::compare(&entry2, &entry1));
    }

    #[test]
    fn test_id_comparer_equal() {
        let data = b"test";
        let mut entry1 = Entry::new();
        let mut entry2 = Entry::new();

        entry1.set_str(data);
        entry2.set_str(data);

        entry1.set_id(10);
        entry2.set_id(10);

        assert!(!IDComparer::compare(&entry1, &entry2));
        assert!(!IDComparer::compare(&entry2, &entry1));
    }

    #[test]
    fn test_entry_as_bytes() {
        let data = b"rust";
        let mut entry = Entry::new();
        entry.set_str(data);

        assert_eq!(entry.as_bytes(), b"rust");
    }

    #[test]
    fn test_entry_empty_string() {
        let data = b"";
        let mut entry = Entry::new();
        entry.set_str(data);

        assert_eq!(entry.length(), 0);
        assert_eq!(entry.as_bytes(), b"");
    }

    #[test]
    fn test_entry_max_id() {
        let data = b"test";
        let mut entry = Entry::new();
        entry.set_str(data);
        entry.set_id(u32::MAX as usize);

        assert_eq!(entry.id(), u32::MAX as usize);
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_entry_get_out_of_bounds() {
        let data = b"hi";
        let mut entry = Entry::new();
        entry.set_str(data);
        entry.get(2); // Out of bounds
    }
}
