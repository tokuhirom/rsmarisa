//! Keyset for collecting keys to build a trie.
//!
//! Ported from:
//! - include/marisa/keyset.h
//! - lib/marisa/keyset.cc

use crate::key::Key;
use std::io;

/// Block sizes for memory allocation.
const BASE_BLOCK_SIZE: usize = 4096;
const EXTRA_BLOCK_SIZE: usize = 1024;
const KEY_BLOCK_SIZE: usize = 256;

/// Keyset collects keys for trie construction.
///
/// Keys are stored in blocks to minimize allocations and provide
/// stable addresses for borrowed string data.
pub struct Keyset {
    /// Blocks of base size for normal string storage.
    base_blocks: Vec<Box<[u8; BASE_BLOCK_SIZE]>>,
    /// Blocks for strings larger than EXTRA_BLOCK_SIZE.
    extra_blocks: Vec<Vec<u8>>,
    /// Blocks of Key objects.
    key_blocks: Vec<Box<[Key; KEY_BLOCK_SIZE]>>,
    /// Current write position in the current base block.
    ptr_offset: usize,
    /// Available space remaining in current base block.
    avail: usize,
    /// Total number of keys.
    size: usize,
    /// Total length of all key strings.
    total_length: usize,
}

impl Default for Keyset {
    fn default() -> Self {
        Self::new()
    }
}

impl Keyset {
    /// Creates a new empty keyset.
    pub fn new() -> Self {
        Keyset {
            base_blocks: Vec::new(),
            extra_blocks: Vec::new(),
            key_blocks: Vec::new(),
            ptr_offset: 0,
            avail: 0,
            size: 0,
            total_length: 0,
        }
    }

    /// Adds a key to the keyset.
    pub fn push_back_key(&mut self, key: &Key) {
        let key_bytes = key.as_bytes();
        let key_ptr = self.reserve(key_bytes.len());

        // Copy string data
        unsafe {
            std::ptr::copy_nonoverlapping(key_bytes.as_ptr(), key_ptr, key_bytes.len());
        }

        // Create new Key in key block
        let key_block_idx = self.size / KEY_BLOCK_SIZE;
        let key_idx = self.size % KEY_BLOCK_SIZE;
        let new_key = &mut self.key_blocks[key_block_idx][key_idx];

        // Set string from our stable storage
        let stored_slice =
            unsafe { std::slice::from_raw_parts(key_ptr as *const u8, key_bytes.len()) };
        new_key.set_bytes(stored_slice);
        new_key.set_id(key.id());

        self.size += 1;
        self.total_length += key_bytes.len();
    }

    /// Adds a key with an end marker character.
    pub fn push_back_key_with_marker(&mut self, key: &Key, end_marker: u8) {
        if self.size / KEY_BLOCK_SIZE == self.key_blocks.len() {
            self.append_key_block();
        }

        let key_bytes = key.as_bytes();
        let total_len = key_bytes.len() + 1;
        let key_ptr = self.reserve(total_len);

        // Copy string data and add marker
        unsafe {
            std::ptr::copy_nonoverlapping(key_bytes.as_ptr(), key_ptr, key_bytes.len());
            *key_ptr.add(key_bytes.len()) = end_marker;
        }

        // Create new Key in key block
        let key_block_idx = self.size / KEY_BLOCK_SIZE;
        let key_idx = self.size % KEY_BLOCK_SIZE;
        let new_key = &mut self.key_blocks[key_block_idx][key_idx];

        // Set string from our stable storage (without end marker)
        let stored_slice =
            unsafe { std::slice::from_raw_parts(key_ptr as *const u8, key_bytes.len()) };
        new_key.set_bytes(stored_slice);
        new_key.set_id(key.id());

        self.size += 1;
        self.total_length += key_bytes.len();
    }

    /// Adds a string to the keyset with default weight of 1.0.
    pub fn push_back_str(&mut self, s: &str) -> io::Result<()> {
        self.push_back_bytes(s.as_bytes(), 1.0)
    }

    /// Adds bytes to the keyset with specified weight.
    pub fn push_back_bytes(&mut self, bytes: &[u8], weight: f32) -> io::Result<()> {
        if bytes.len() > u32::MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Key too long",
            ));
        }

        let key_ptr = self.reserve(bytes.len());

        // Copy string data
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), key_ptr, bytes.len());
        }

        // Create new Key in key block
        let key_block_idx = self.size / KEY_BLOCK_SIZE;
        let key_idx = self.size % KEY_BLOCK_SIZE;
        let key = &mut self.key_blocks[key_block_idx][key_idx];

        // Set string from our stable storage
        let stored_slice = unsafe { std::slice::from_raw_parts(key_ptr as *const u8, bytes.len()) };
        key.set_bytes(stored_slice);
        key.set_weight(weight);

        self.size += 1;
        self.total_length += bytes.len();

        Ok(())
    }

    /// Returns a reference to the key at the specified index.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    pub fn get(&self, i: usize) -> &Key {
        assert!(i < self.size, "Index out of bounds");
        &self.key_blocks[i / KEY_BLOCK_SIZE][i % KEY_BLOCK_SIZE]
    }

    /// Returns a mutable reference to the key at the specified index.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    pub fn get_mut(&mut self, i: usize) -> &mut Key {
        assert!(i < self.size, "Index out of bounds");
        &mut self.key_blocks[i / KEY_BLOCK_SIZE][i % KEY_BLOCK_SIZE]
    }

    /// Returns the number of keys in the keyset.
    pub fn num_keys(&self) -> usize {
        self.size
    }

    /// Returns true if the keyset is empty.
    pub fn empty(&self) -> bool {
        self.size == 0
    }

    /// Returns the number of keys (alias for num_keys).
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the total length of all key strings.
    pub fn total_length(&self) -> usize {
        self.total_length
    }

    /// Resets the keyset to reuse allocated memory.
    pub fn reset(&mut self) {
        self.ptr_offset = 0;
        self.avail = 0;
        self.size = 0;
        self.total_length = 0;
        // Keep allocated blocks for reuse
    }

    /// Clears all data and frees memory.
    pub fn clear(&mut self) {
        *self = Keyset::new();
    }

    /// Swaps with another keyset.
    pub fn swap(&mut self, other: &mut Keyset) {
        std::mem::swap(self, other);
    }

    /// Reserves space for a string of the given size.
    ///
    /// Returns a mutable pointer to the reserved space.
    fn reserve(&mut self, size: usize) -> *mut u8 {
        // Ensure we have a key block for the new key
        if self.size / KEY_BLOCK_SIZE == self.key_blocks.len() {
            self.append_key_block();
        }

        // For large strings, use an extra block
        if size > EXTRA_BLOCK_SIZE {
            self.append_extra_block(size);
            return self.extra_blocks.last_mut().unwrap().as_mut_ptr();
        }

        // Need a new base block?
        if size > self.avail {
            self.append_base_block();
        }

        // Get pointer to available space
        let block_idx = self.base_blocks.len() - 1;
        let ptr = unsafe { self.base_blocks[block_idx].as_mut_ptr().add(self.ptr_offset) };

        self.ptr_offset += size;
        self.avail -= size;

        ptr
    }

    /// Appends a new base block for string storage.
    fn append_base_block(&mut self) {
        self.base_blocks.push(Box::new([0u8; BASE_BLOCK_SIZE]));
        self.ptr_offset = 0;
        self.avail = BASE_BLOCK_SIZE;
    }

    /// Appends a new extra block for large string storage.
    fn append_extra_block(&mut self, size: usize) {
        self.extra_blocks.push(vec![0u8; size]);
    }

    /// Appends a new key block.
    fn append_key_block(&mut self) {
        // Create a new block with default Keys
        let block = Box::new([(); KEY_BLOCK_SIZE].map(|_| Key::new()));
        self.key_blocks.push(block);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyset_new() {
        let keyset = Keyset::new();
        assert_eq!(keyset.size(), 0);
        assert_eq!(keyset.total_length(), 0);
        assert!(keyset.empty());
    }

    #[test]
    fn test_keyset_default() {
        let keyset = Keyset::default();
        assert_eq!(keyset.size(), 0);
    }

    #[test]
    fn test_keyset_push_back_str() {
        let mut keyset = Keyset::new();

        keyset.push_back_str("hello").unwrap();
        keyset.push_back_str("world").unwrap();

        assert_eq!(keyset.size(), 2);
        assert_eq!(keyset.total_length(), 10);
        assert_eq!(keyset.get(0).as_str(), "hello");
        assert_eq!(keyset.get(1).as_str(), "world");
    }

    #[test]
    fn test_keyset_push_back_bytes() {
        let mut keyset = Keyset::new();

        keyset.push_back_bytes(b"test", 2.5).unwrap();

        assert_eq!(keyset.size(), 1);
        assert_eq!(keyset.get(0).as_bytes(), b"test");
        assert!((keyset.get(0).weight() - 2.5).abs() < 0.001);
    }

    #[test]
    fn test_keyset_push_back_key() {
        let mut keyset = Keyset::new();
        let s = "example";

        let mut key = Key::new();
        key.set_str(s);
        key.set_id(42);

        keyset.push_back_key(&key);

        assert_eq!(keyset.size(), 1);
        assert_eq!(keyset.get(0).as_str(), "example");
        assert_eq!(keyset.get(0).id(), 42);
    }

    #[test]
    fn test_keyset_push_back_key_with_marker() {
        let mut keyset = Keyset::new();
        let s = "test";

        let mut key = Key::new();
        key.set_str(s);
        key.set_id(10);

        keyset.push_back_key_with_marker(&key, b'\0');

        assert_eq!(keyset.size(), 1);
        assert_eq!(keyset.get(0).as_str(), "test");
        // End marker is not included in the key length
    }

    #[test]
    fn test_keyset_get_mut() {
        let mut keyset = Keyset::new();
        keyset.push_back_str("hello").unwrap();

        {
            let key = keyset.get_mut(0);
            key.set_id(99);
        }

        assert_eq!(keyset.get(0).id(), 99);
    }

    #[test]
    fn test_keyset_reset() {
        let mut keyset = Keyset::new();
        keyset.push_back_str("hello").unwrap();
        keyset.push_back_str("world").unwrap();

        assert_eq!(keyset.size(), 2);

        keyset.reset();

        assert_eq!(keyset.size(), 0);
        assert_eq!(keyset.total_length(), 0);
    }

    #[test]
    fn test_keyset_clear() {
        let mut keyset = Keyset::new();
        keyset.push_back_str("hello").unwrap();

        keyset.clear();

        assert_eq!(keyset.size(), 0);
        assert_eq!(keyset.total_length(), 0);
    }

    #[test]
    fn test_keyset_swap() {
        let mut ks1 = Keyset::new();
        ks1.push_back_str("one").unwrap();

        let mut ks2 = Keyset::new();
        ks2.push_back_str("two").unwrap();
        ks2.push_back_str("three").unwrap();

        ks1.swap(&mut ks2);

        assert_eq!(ks1.size(), 2);
        assert_eq!(ks1.get(0).as_str(), "two");
        assert_eq!(ks2.size(), 1);
        assert_eq!(ks2.get(0).as_str(), "one");
    }

    #[test]
    fn test_keyset_many_keys() {
        let mut keyset = Keyset::new();

        // Push enough keys to trigger multiple blocks
        for i in 0..1000 {
            keyset.push_back_str(&format!("key{}", i)).unwrap();
        }

        assert_eq!(keyset.size(), 1000);

        // Verify some keys
        assert_eq!(keyset.get(0).as_str(), "key0");
        assert_eq!(keyset.get(500).as_str(), "key500");
        assert_eq!(keyset.get(999).as_str(), "key999");
    }

    #[test]
    fn test_keyset_large_string() {
        let mut keyset = Keyset::new();

        // String larger than EXTRA_BLOCK_SIZE
        let large_str = "x".repeat(2000);
        keyset.push_back_str(&large_str).unwrap();

        assert_eq!(keyset.size(), 1);
        assert_eq!(keyset.get(0).as_str(), large_str);
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_keyset_get_out_of_bounds() {
        let keyset = Keyset::new();
        keyset.get(0);
    }

    #[test]
    fn test_keyset_empty() {
        let mut keyset = Keyset::new();
        assert!(keyset.empty());

        keyset.push_back_str("test").unwrap();
        assert!(!keyset.empty());
    }
}
