//! Key type for trie operations.
//!
//! Ported from: include/marisa/key.h
//!
//! This is the public API Key type, distinct from grimoire::trie::Key
//! which is used internally.

use std::fmt;

/// Union type to hold either an ID or a weight.
#[derive(Clone, Copy)]
union KeyUnion {
    /// Key ID for indexed access.
    id: u32,
    /// Weight for weighted keys.
    weight: f32,
}

impl Default for KeyUnion {
    fn default() -> Self {
        KeyUnion { id: 0 }
    }
}

/// Key represents a dictionary key with its string and metadata.
///
/// A key contains:
/// - A string (ptr + length)
/// - Either an ID or a weight (union)
#[derive(Clone)]
pub struct Key {
    /// Pointer to key string data (borrowed).
    ptr: Option<*const u8>,
    /// Length of key string.
    length: u32,
    /// Union holding either ID or weight.
    union: KeyUnion,
}

// Manual Debug implementation since raw pointers don't implement Debug
impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Key")
            .field("ptr", &self.ptr.map(|_| "..."))
            .field("length", &self.length)
            .field("id_or_weight", &unsafe { self.union.id })
            .finish()
    }
}

impl Default for Key {
    fn default() -> Self {
        Self::new()
    }
}

impl Key {
    /// Creates a new empty key.
    pub fn new() -> Self {
        Key {
            ptr: None,
            length: 0,
            union: KeyUnion::default(),
        }
    }

    /// Returns the character at the specified index.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    pub fn get(&self, i: usize) -> u8 {
        assert!((i as u32) < self.length, "Index out of bounds");
        if let Some(ptr) = self.ptr {
            unsafe { *ptr.add(i) }
        } else {
            panic!("Key has no string data");
        }
    }

    /// Sets the key from a string slice.
    pub fn set_str(&mut self, s: &str) {
        assert!(s.len() <= u32::MAX as usize, "String too long");
        self.ptr = Some(s.as_ptr());
        self.length = s.len() as u32;
    }

    /// Sets the key from a byte slice.
    pub fn set_bytes(&mut self, bytes: &[u8]) {
        assert!(bytes.len() <= u32::MAX as usize, "Bytes too long");
        if bytes.is_empty() {
            self.ptr = None;
            self.length = 0;
        } else {
            self.ptr = Some(bytes.as_ptr());
            self.length = bytes.len() as u32;
        }
    }

    /// Sets the key ID.
    pub fn set_id(&mut self, id: usize) {
        assert!(id <= u32::MAX as usize, "ID too large");
        self.union = KeyUnion { id: id as u32 };
    }

    /// Sets the key weight.
    pub fn set_weight(&mut self, weight: f32) {
        self.union = KeyUnion { weight };
    }

    /// Returns the key as a byte slice.
    ///
    /// Returns an empty slice if no string is set.
    pub fn as_bytes(&self) -> &[u8] {
        if let Some(ptr) = self.ptr {
            unsafe { std::slice::from_raw_parts(ptr, self.length as usize) }
        } else {
            &[]
        }
    }

    /// Returns the key string as a str reference.
    ///
    /// # Panics
    ///
    /// Panics if the key contains invalid UTF-8.
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(self.as_bytes()).expect("Invalid UTF-8 in key")
    }

    /// Returns a pointer to the key data.
    pub fn ptr(&self) -> Option<*const u8> {
        self.ptr
    }

    /// Returns the length of the key string.
    pub fn length(&self) -> usize {
        self.length as usize
    }

    /// Returns the key ID.
    ///
    /// # Safety
    ///
    /// This accesses the union as an ID. The caller must ensure
    /// that set_id() was called more recently than set_weight().
    pub fn id(&self) -> usize {
        unsafe { self.union.id as usize }
    }

    /// Returns the key weight.
    ///
    /// # Safety
    ///
    /// This accesses the union as a weight. The caller must ensure
    /// that set_weight() was called more recently than set_id().
    pub fn weight(&self) -> f32 {
        unsafe { self.union.weight }
    }

    /// Clears the key to empty state.
    pub fn clear(&mut self) {
        *self = Key::new();
    }

    /// Swaps with another key.
    pub fn swap(&mut self, other: &mut Key) {
        std::mem::swap(self, other);
    }
}

// Safety: Key only holds a pointer that must remain valid for its lifetime.
// The user is responsible for ensuring the borrowed data outlives the Key.
unsafe impl Send for Key {}
unsafe impl Sync for Key {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_new() {
        let key = Key::new();
        assert_eq!(key.length(), 0);
        assert_eq!(key.id(), 0);
        assert_eq!(key.as_bytes(), &[]);
    }

    #[test]
    fn test_key_default() {
        let key = Key::default();
        assert_eq!(key.length(), 0);
    }

    #[test]
    fn test_key_set_str() {
        let s = "hello";
        let mut key = Key::new();
        key.set_str(s);

        assert_eq!(key.length(), 5);
        assert_eq!(key.as_str(), "hello");
        assert_eq!(key.as_bytes(), b"hello");
    }

    #[test]
    fn test_key_set_bytes() {
        let bytes = b"world";
        let mut key = Key::new();
        key.set_bytes(bytes);

        assert_eq!(key.length(), 5);
        assert_eq!(key.as_bytes(), b"world");
    }

    #[test]
    fn test_key_set_empty_bytes() {
        let mut key = Key::new();
        key.set_str("test");
        key.set_bytes(&[]);

        assert_eq!(key.length(), 0);
        assert_eq!(key.as_bytes(), &[]);
    }

    #[test]
    fn test_key_get() {
        let s = "test";
        let mut key = Key::new();
        key.set_str(s);

        assert_eq!(key.get(0), b't');
        assert_eq!(key.get(1), b'e');
        assert_eq!(key.get(2), b's');
        assert_eq!(key.get(3), b't');
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_key_get_out_of_bounds() {
        let s = "test";
        let mut key = Key::new();
        key.set_str(s);
        key.get(4);
    }

    #[test]
    fn test_key_set_id() {
        let mut key = Key::new();
        key.set_id(42);
        assert_eq!(key.id(), 42);
    }

    #[test]
    fn test_key_set_weight() {
        let mut key = Key::new();
        key.set_weight(3.15);
        assert!((key.weight() - 3.15).abs() < 0.001);
    }

    #[test]
    fn test_key_id_weight_union() {
        let mut key = Key::new();

        // Set as ID
        key.set_id(100);
        assert_eq!(key.id(), 100);

        // Overwrite with weight
        key.set_weight(2.5);
        assert!((key.weight() - 2.5).abs() < 0.001);
    }

    #[test]
    fn test_key_clear() {
        let mut key = Key::new();
        key.set_str("test");
        key.set_id(10);

        key.clear();

        assert_eq!(key.length(), 0);
        assert_eq!(key.id(), 0);
        assert_eq!(key.as_bytes(), &[]);
    }

    #[test]
    fn test_key_swap() {
        let s1 = "hello";
        let s2 = "world";

        let mut k1 = Key::new();
        k1.set_str(s1);
        k1.set_id(1);

        let mut k2 = Key::new();
        k2.set_str(s2);
        k2.set_id(2);

        k1.swap(&mut k2);

        assert_eq!(k1.as_str(), "world");
        assert_eq!(k1.id(), 2);
        assert_eq!(k2.as_str(), "hello");
        assert_eq!(k2.id(), 1);
    }

    #[test]
    fn test_key_with_unicode() {
        let s = "こんにちは";
        let mut key = Key::new();
        key.set_str(s);

        assert_eq!(key.length(), s.len());
        assert_eq!(key.as_str(), "こんにちは");
    }

    // Note: Cannot safely test set_str with length > u32::MAX
    // as creating such a string would require invalid operations.
}
