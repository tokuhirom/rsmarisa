//! Key structures for trie operations.
//!
//! Ported from: lib/marisa/grimoire/trie/key.h
//!
//! This module provides Key and ReverseKey structures that represent
//! string keys during trie construction and queries, along with associated
//! metadata like weight, terminal position, or key ID.

/// Metadata associated with a key (either weight or terminal position).
#[derive(Debug, Clone, Copy, PartialEq)]
enum WeightOrTerminal {
    /// Weight value for weighted sorting.
    Weight(f32),
    /// Terminal position in the trie.
    Terminal(u32),
}

impl Default for WeightOrTerminal {
    fn default() -> Self {
        WeightOrTerminal::Terminal(0)
    }
}

/// Key representing a forward string with metadata.
///
/// Key holds a borrowed slice of bytes representing a string, along with
/// optional metadata like weight (for frequency-based sorting) or terminal
/// position (for trie node identification).
#[derive(Debug, Clone, Copy)]
pub struct Key<'a> {
    /// Byte slice representing the key string.
    bytes: &'a [u8],
    /// Weight or terminal position.
    union: WeightOrTerminal,
    /// Key ID.
    id: u32,
}

impl<'a> Default for Key<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Key<'a> {
    /// Creates a new empty key.
    pub fn new() -> Self {
        Key {
            bytes: &[],
            union: WeightOrTerminal::default(),
            id: 0,
        }
    }

    /// Returns the byte at the given index.
    ///
    /// # Arguments
    ///
    /// * `i` - Index into the key string
    ///
    /// # Panics
    ///
    /// Panics if i >= length()
    #[inline]
    pub fn get(&self, i: usize) -> u8 {
        assert!(i < self.bytes.len(), "Index out of bounds");
        self.bytes[i]
    }

    /// Creates a substring of this key.
    ///
    /// # Arguments
    ///
    /// * `pos` - Starting position
    /// * `length` - Length of substring
    ///
    /// # Panics
    ///
    /// Panics if pos or length are out of bounds
    pub fn substr(&mut self, pos: usize, length: usize) {
        assert!(pos <= self.bytes.len(), "pos out of bounds");
        assert!(length <= self.bytes.len(), "length out of bounds");
        assert!(pos <= self.bytes.len() - length, "substring out of bounds");
        self.bytes = &self.bytes[pos..pos + length];
    }

    /// Sets the string content.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Byte slice to use as the key
    ///
    /// # Panics
    ///
    /// Panics if bytes.len() > u32::MAX
    pub fn set_str(&mut self, bytes: &'a [u8]) {
        assert!(bytes.len() <= u32::MAX as usize, "String too long");
        self.bytes = bytes;
    }

    /// Sets the weight.
    #[inline]
    pub fn set_weight(&mut self, weight: f32) {
        self.union = WeightOrTerminal::Weight(weight);
    }

    /// Sets the terminal position.
    ///
    /// # Panics
    ///
    /// Panics if terminal > u32::MAX
    #[inline]
    pub fn set_terminal(&mut self, terminal: usize) {
        assert!(terminal <= u32::MAX as usize, "Terminal too large");
        self.union = WeightOrTerminal::Terminal(terminal as u32);
    }

    /// Sets the key ID.
    ///
    /// # Panics
    ///
    /// Panics if id > u32::MAX
    #[inline]
    pub fn set_id(&mut self, id: usize) {
        assert!(id <= u32::MAX as usize, "ID too large");
        self.id = id as u32;
    }

    /// Returns a pointer to the string data.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes
    }

    /// Returns the length of the key.
    #[inline]
    pub fn length(&self) -> usize {
        self.bytes.len()
    }

    /// Returns the weight (if set).
    ///
    /// # Panics
    ///
    /// Panics if terminal was set instead of weight
    #[inline]
    pub fn weight(&self) -> f32 {
        match self.union {
            WeightOrTerminal::Weight(w) => w,
            _ => panic!("Weight not set"),
        }
    }

    /// Returns the terminal position (if set).
    ///
    /// # Panics
    ///
    /// Panics if weight was set instead of terminal
    #[inline]
    pub fn terminal(&self) -> usize {
        match self.union {
            WeightOrTerminal::Terminal(t) => t as usize,
            _ => panic!("Terminal not set"),
        }
    }

    /// Returns the key ID.
    #[inline]
    pub fn id(&self) -> usize {
        self.id as usize
    }
}

impl<'a> PartialEq for Key<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl<'a> Eq for Key<'a> {}

impl<'a> PartialOrd for Key<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for Key<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Lexicographic comparison treating bytes as unsigned
        self.bytes.cmp(other.bytes)
    }
}

impl<'a> crate::grimoire::algorithm::sort::Sortable for Key<'a> {
    fn get(&self, index: usize) -> Option<u8> {
        if index < self.bytes.len() {
            Some(self.bytes[index])
        } else {
            None
        }
    }

    fn length(&self) -> usize {
        self.bytes.len()
    }
}

/// Reverse key representing a string accessed in reverse order.
///
/// ReverseKey is similar to Key but accesses the string bytes in reverse
/// order. This is useful for certain trie construction algorithms.
#[derive(Debug, Clone, Copy)]
pub struct ReverseKey<'a> {
    /// Full byte slice (stored normally).
    bytes: &'a [u8],
    /// End position (one past the last byte to access).
    end: usize,
    /// Current view length.
    length: usize,
    /// Weight or terminal position.
    union: WeightOrTerminal,
    /// Key ID.
    id: u32,
}

impl<'a> Default for ReverseKey<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ReverseKey<'a> {
    /// Creates a new empty reverse key.
    pub fn new() -> Self {
        ReverseKey {
            bytes: &[],
            end: 0,
            length: 0,
            union: WeightOrTerminal::default(),
            id: 0,
        }
    }

    /// Returns the byte at the given reverse index.
    ///
    /// # Arguments
    ///
    /// * `i` - Reverse index (0 = last byte accessed)
    ///
    /// # Panics
    ///
    /// Panics if i >= length()
    #[inline]
    pub fn get(&self, i: usize) -> u8 {
        assert!(i < self.length, "Index out of bounds");
        self.bytes[self.end - i - 1]
    }

    /// Creates a reverse substring.
    ///
    /// # Arguments
    ///
    /// * `pos` - Starting position from the current end
    /// * `length` - Length of substring
    ///
    /// # Panics
    ///
    /// Panics if pos or length are out of bounds
    pub fn substr(&mut self, pos: usize, length: usize) {
        assert!(pos <= self.length, "pos out of bounds");
        assert!(length <= self.length, "length out of bounds");
        assert!(pos <= self.length - length, "substring out of bounds");
        self.end -= pos;
        self.length = length;
    }

    /// Sets the string content.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Byte slice to use as the key
    ///
    /// # Panics
    ///
    /// Panics if bytes.len() > u32::MAX
    pub fn set_str(&mut self, bytes: &'a [u8]) {
        assert!(bytes.len() <= u32::MAX as usize, "String too long");
        self.bytes = bytes;
        self.end = bytes.len();
        self.length = bytes.len();
    }

    /// Sets the weight.
    #[inline]
    pub fn set_weight(&mut self, weight: f32) {
        self.union = WeightOrTerminal::Weight(weight);
    }

    /// Sets the terminal position.
    ///
    /// # Panics
    ///
    /// Panics if terminal > u32::MAX
    #[inline]
    pub fn set_terminal(&mut self, terminal: usize) {
        assert!(terminal <= u32::MAX as usize, "Terminal too large");
        self.union = WeightOrTerminal::Terminal(terminal as u32);
    }

    /// Sets the key ID.
    ///
    /// # Panics
    ///
    /// Panics if id > u32::MAX
    #[inline]
    pub fn set_id(&mut self, id: usize) {
        assert!(id <= u32::MAX as usize, "ID too large");
        self.id = id as u32;
    }

    /// Returns the forward byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[self.end - self.length..self.end]
    }

    /// Returns the length of the key.
    #[inline]
    pub fn length(&self) -> usize {
        self.length
    }

    /// Returns the weight (if set).
    ///
    /// # Panics
    ///
    /// Panics if terminal was set instead of weight
    #[inline]
    pub fn weight(&self) -> f32 {
        match self.union {
            WeightOrTerminal::Weight(w) => w,
            _ => panic!("Weight not set"),
        }
    }

    /// Returns the terminal position (if set).
    ///
    /// # Panics
    ///
    /// Panics if weight was set instead of terminal
    #[inline]
    pub fn terminal(&self) -> usize {
        match self.union {
            WeightOrTerminal::Terminal(t) => t as usize,
            _ => panic!("Terminal not set"),
        }
    }

    /// Returns the key ID.
    #[inline]
    pub fn id(&self) -> usize {
        self.id as usize
    }
}

impl<'a> PartialEq for ReverseKey<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.length != other.length {
            return false;
        }
        for i in 0..self.length {
            if self.get(i) != other.get(i) {
                return false;
            }
        }
        true
    }
}

impl<'a> Eq for ReverseKey<'a> {}

impl<'a> PartialOrd for ReverseKey<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for ReverseKey<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        for i in 0..self.length {
            if i == other.length {
                return std::cmp::Ordering::Greater;
            }
            match self.get(i).cmp(&other.get(i)) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }
        self.length.cmp(&other.length)
    }
}

impl<'a> crate::grimoire::algorithm::sort::Sortable for ReverseKey<'a> {
    fn get(&self, index: usize) -> Option<u8> {
        if index < self.length {
            Some(ReverseKey::get(self, index))
        } else {
            None
        }
    }

    fn length(&self) -> usize {
        self.length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_new() {
        let key = Key::new();
        assert_eq!(key.length(), 0);
        assert_eq!(key.id(), 0);
    }

    #[test]
    fn test_key_set_str() {
        let data = b"hello";
        let mut key = Key::new();
        key.set_str(data);

        assert_eq!(key.length(), 5);
        assert_eq!(key.get(0), b'h');
        assert_eq!(key.get(4), b'o');
    }

    #[test]
    fn test_key_substr() {
        let data = b"hello world";
        let mut key = Key::new();
        key.set_str(data);
        key.substr(6, 5); // "world"

        assert_eq!(key.length(), 5);
        assert_eq!(key.get(0), b'w');
        assert_eq!(key.get(4), b'd');
    }

    #[test]
    fn test_key_weight_terminal() {
        let data = b"test";
        let mut key = Key::new();
        key.set_str(data);

        key.set_weight(3.15);
        assert_eq!(key.weight(), 3.15);

        key.set_terminal(42);
        assert_eq!(key.terminal(), 42);
    }

    #[test]
    fn test_key_id() {
        let data = b"test";
        let mut key = Key::new();
        key.set_str(data);
        key.set_id(123);

        assert_eq!(key.id(), 123);
    }

    #[test]
    fn test_key_equality() {
        let data1 = b"hello";
        let data2 = b"hello";
        let data3 = b"world";

        let mut key1 = Key::new();
        let mut key2 = Key::new();
        let mut key3 = Key::new();

        key1.set_str(data1);
        key2.set_str(data2);
        key3.set_str(data3);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_key_ordering() {
        let data1 = b"apple";
        let data2 = b"banana";
        let data3 = b"cherry";

        let mut key1 = Key::new();
        let mut key2 = Key::new();
        let mut key3 = Key::new();

        key1.set_str(data1);
        key2.set_str(data2);
        key3.set_str(data3);

        assert!(key1 < key2);
        assert!(key2 < key3);
        assert!(key1 < key3);
    }

    #[test]
    fn test_reverse_key_new() {
        let key = ReverseKey::new();
        assert_eq!(key.length(), 0);
        assert_eq!(key.id(), 0);
    }

    #[test]
    fn test_reverse_key_set_str() {
        let data = b"hello";
        let mut key = ReverseKey::new();
        key.set_str(data);

        assert_eq!(key.length(), 5);
        assert_eq!(key.get(0), b'o'); // Reverse: last char first
        assert_eq!(key.get(4), b'h'); // First char last
    }

    #[test]
    fn test_reverse_key_substr() {
        let data = b"hello";
        let mut key = ReverseKey::new();
        key.set_str(data);
        // Original: o, l, l, e, h (in reverse access order)
        // substr(1, 3) skips first (o) and takes 3: l, l, e
        key.substr(1, 3);

        assert_eq!(key.length(), 3);
        assert_eq!(key.get(0), b'l'); // First of remaining
        assert_eq!(key.get(1), b'l'); // Second
        assert_eq!(key.get(2), b'e'); // Third
    }

    #[test]
    fn test_reverse_key_equality() {
        let data1 = b"hello";
        let data2 = b"hello";
        let data3 = b"world";

        let mut key1 = ReverseKey::new();
        let mut key2 = ReverseKey::new();
        let mut key3 = ReverseKey::new();

        key1.set_str(data1);
        key2.set_str(data2);
        key3.set_str(data3);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_reverse_key_ordering() {
        let data1 = b"apple";
        let data2 = b"elppa"; // Same as apple reversed

        let mut key1 = ReverseKey::new();
        let mut key2 = ReverseKey::new();

        key1.set_str(data1);
        key2.set_str(data2);

        // In reverse, "apple" is "elppa" vs "elppa" reversed is "apple"
        // So they should compare based on reverse order
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_key_as_bytes() {
        let data = b"test";
        let mut key = Key::new();
        key.set_str(data);

        assert_eq!(key.as_bytes(), data);
    }

    #[test]
    fn test_reverse_key_as_bytes() {
        let data = b"test";
        let mut key = ReverseKey::new();
        key.set_str(data);

        assert_eq!(key.as_bytes(), data);
    }
}
