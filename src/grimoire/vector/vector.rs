//! Generic vector wrapper.
//!
//! Ported from: lib/marisa/grimoire/vector/vector.h
//!
//! This module provides a custom vector implementation that supports
//! serialization and memory mapping operations.

use crate::grimoire::io::{Mapper, Reader, Writer};

/// Generic vector for internal use with serialization support.
///
/// This vector is similar to std::Vec but with additional features
/// for memory mapping and serialization. It uses Copy/Clone trait
/// bounds to ensure safe serialization.
pub struct Vector<T: Copy> {
    data: Vec<T>,
    fixed: bool,
}

impl<T: Copy> Vector<T> {
    /// Creates a new empty vector.
    #[inline]
    pub fn new() -> Self {
        Vector {
            data: Vec::new(),
            fixed: false,
        }
    }

    /// Pushes a value onto the end of the vector.
    ///
    /// # Panics
    ///
    /// Panics if the vector is fixed.
    #[inline]
    pub fn push_back(&mut self, value: T) {
        assert!(!self.fixed, "Cannot modify fixed vector");
        self.data.push(value);
    }

    /// Removes the last element from the vector.
    ///
    /// # Panics
    ///
    /// Panics if the vector is empty or fixed.
    #[inline]
    pub fn pop_back(&mut self) {
        assert!(!self.fixed, "Cannot modify fixed vector");
        assert!(!self.data.is_empty(), "Cannot pop from empty vector");
        self.data.pop();
    }

    /// Resizes the vector to the given size, filling with default values.
    ///
    /// # Panics
    ///
    /// Panics if the vector is fixed.
    #[inline]
    pub fn resize(&mut self, size: usize, value: T) {
        assert!(!self.fixed, "Cannot modify fixed vector");
        self.data.resize(size, value);
    }

    /// Reserves capacity for at least `additional` more elements.
    ///
    /// # Panics
    ///
    /// Panics if the vector is fixed.
    #[inline]
    pub fn reserve(&mut self, capacity: usize) {
        assert!(!self.fixed, "Cannot modify fixed vector");
        self.data.reserve(capacity);
    }

    /// Shrinks the capacity to match the size.
    #[inline]
    pub fn shrink(&mut self) {
        assert!(!self.fixed, "Cannot modify fixed vector");
        self.data.shrink_to_fit();
    }

    /// Fixes the vector, preventing further modifications.
    #[inline]
    pub fn fix(&mut self) {
        self.fixed = true;
    }

    /// Returns the number of elements in the vector.
    #[inline]
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Returns the capacity of the vector.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Returns true if the vector is empty.
    #[inline]
    pub fn empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns true if the vector is fixed.
    #[inline]
    pub fn fixed(&self) -> bool {
        self.fixed
    }

    /// Returns the total size in bytes.
    #[inline]
    pub fn total_size(&self) -> usize {
        std::mem::size_of::<T>() * self.data.len()
    }

    /// Returns the I/O size needed for serialization.
    #[inline]
    pub fn io_size(&self) -> usize {
        std::mem::size_of::<u64>() + ((self.total_size() + 7) & !0x07)
    }

    /// Accesses an element by index (const version).
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    /// Accesses an element by index (mutable version).
    ///
    /// # Panics
    ///
    /// Panics if the vector is fixed.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        assert!(!self.fixed, "Cannot modify fixed vector");
        self.data.get_mut(index)
    }

    /// Returns a reference to the last element, or None if empty.
    #[inline]
    pub fn back(&self) -> Option<&T> {
        self.data.last()
    }

    /// Returns a mutable reference to the last element, or None if empty.
    #[inline]
    pub fn back_mut(&mut self) -> Option<&mut T> {
        assert!(!self.fixed, "Cannot modify fixed vector");
        self.data.last_mut()
    }

    /// Returns the vector as an immutable slice.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Returns the vector as a mutable slice.
    ///
    /// # Panics
    ///
    /// Panics if the vector is fixed.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        assert!(!self.fixed, "Cannot modify fixed vector");
        &mut self.data
    }

    /// Clears the vector.
    #[inline]
    pub fn clear(&mut self) {
        *self = Vector::new();
    }

    /// Swaps the contents of two vectors.
    #[inline]
    pub fn swap(&mut self, other: &mut Vector<T>) {
        std::mem::swap(&mut self.data, &mut other.data);
        std::mem::swap(&mut self.fixed, &mut other.fixed);
    }

    /// Maps the vector from a mapper (stub for now).
    pub fn map(&mut self, _mapper: &mut Mapper<'_>) {
        // TODO: implement memory mapping
    }

    /// Reads the vector from a reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - Reader to read from
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails.
    pub fn read(&mut self, reader: &mut Reader) -> std::io::Result<()> {
        // Read the total size (u64)
        let total_size: u64 = reader.read()?;

        // Calculate number of elements
        let elem_size = std::mem::size_of::<T>();
        if elem_size == 0 {
            return Ok(()); // Zero-sized types
        }

        let size = (total_size as usize) / elem_size;

        // Allocate and read elements
        self.data.clear();
        self.data.reserve(size);
        #[allow(clippy::uninit_vec)]
        unsafe {
            self.data.set_len(size);
        }

        if size > 0 {
            reader.read_slice(&mut self.data[..])?;
        }

        // Skip alignment padding
        let padding = ((8 - (total_size % 8)) % 8) as usize;
        if padding > 0 {
            reader.seek(padding)?;
        }

        Ok(())
    }

    /// Writes the vector to a writer.
    ///
    /// Format:
    /// - u64: total_size (size in bytes)
    /// - [T; size]: array elements
    /// - padding: 8-byte alignment padding
    ///
    /// # Arguments
    ///
    /// * `writer` - Writer to write to
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn write(&self, writer: &mut Writer) -> std::io::Result<()> {
        // Write total size as u64
        let total = self.total_size() as u64;
        writer.write(&total)?;

        // Write array elements
        if !self.data.is_empty() {
            writer.write_slice(&self.data[..])?;
        }

        // Write alignment padding to 8 bytes
        let padding = ((8 - (total % 8)) % 8) as usize;
        if padding > 0 {
            writer.seek(padding)?;
        }

        Ok(())
    }
}

impl<T: Copy> Default for Vector<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy> std::ops::Index<usize> for Vector<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T: Copy> std::ops::IndexMut<usize> for Vector<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(!self.fixed, "Cannot modify fixed vector");
        &mut self.data[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_new() {
        let vec: Vector<i32> = Vector::new();
        assert_eq!(vec.size(), 0);
        assert!(vec.empty());
    }

    #[test]
    fn test_vector_push_pop() {
        let mut vec = Vector::new();
        vec.push_back(1);
        vec.push_back(2);
        vec.push_back(3);

        assert_eq!(vec.size(), 3);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 3);

        vec.pop_back();
        assert_eq!(vec.size(), 2);
    }

    #[test]
    fn test_vector_resize() {
        let mut vec = Vector::new();
        vec.resize(5, 42);

        assert_eq!(vec.size(), 5);
        for i in 0..5 {
            assert_eq!(vec[i], 42);
        }
    }

    #[test]
    fn test_vector_fix() {
        let mut vec = Vector::new();
        vec.push_back(1);
        vec.fix();

        assert!(vec.fixed());
    }

    #[test]
    #[should_panic(expected = "Cannot modify fixed vector")]
    fn test_vector_fixed_push() {
        let mut vec = Vector::new();
        vec.fix();
        vec.push_back(1);
    }

    #[test]
    fn test_vector_write_read() {
        // Rust-specific: Test Vector<T> serialization
        use crate::grimoire::io::{Reader, Writer};

        let mut vec = Vector::new();
        vec.push_back(1u32);
        vec.push_back(2u32);
        vec.push_back(3u32);

        // Write to buffer
        let mut writer = Writer::from_vec(Vec::new());
        vec.write(&mut writer).unwrap();

        let data = writer.into_inner().unwrap();

        // Read back
        let mut reader = Reader::from_bytes(&data);
        let mut vec2: Vector<u32> = Vector::new();
        vec2.read(&mut reader).unwrap();

        assert_eq!(vec2.size(), 3);
        assert_eq!(vec2[0], 1);
        assert_eq!(vec2[1], 2);
        assert_eq!(vec2[2], 3);
    }

    #[test]
    fn test_vector_write_read_empty() {
        // Rust-specific: Test empty Vector<T> serialization
        use crate::grimoire::io::{Reader, Writer};

        let vec: Vector<u64> = Vector::new();

        // Write to buffer
        let mut writer = Writer::from_vec(Vec::new());
        vec.write(&mut writer).unwrap();

        let data = writer.into_inner().unwrap();

        // Read back
        let mut reader = Reader::from_bytes(&data);
        let mut vec2: Vector<u64> = Vector::new();
        vec2.read(&mut reader).unwrap();

        assert_eq!(vec2.size(), 0);
        assert!(vec2.empty());
    }

    #[test]
    fn test_vector_write_alignment() {
        // Rust-specific: Test 8-byte alignment padding
        use crate::grimoire::io::Writer;

        let mut vec = Vector::new();
        vec.push_back(1u8);
        vec.push_back(2u8);
        vec.push_back(3u8);

        let mut writer = Writer::from_vec(Vec::new());
        vec.write(&mut writer).unwrap();

        let data = writer.into_inner().unwrap();

        // Format: 8 bytes (u64 total_size) + 3 bytes (data) + 5 bytes (padding) = 16 bytes
        assert_eq!(data.len(), 16);

        // Check total_size field (first 8 bytes = u64)
        let total_size = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        assert_eq!(total_size, 3); // 3 bytes total

        // Check data
        assert_eq!(data[8], 1);
        assert_eq!(data[9], 2);
        assert_eq!(data[10], 3);

        // Check padding (should be zeros)
        assert_eq!(data[11], 0);
        assert_eq!(data[12], 0);
        assert_eq!(data[13], 0);
        assert_eq!(data[14], 0);
        assert_eq!(data[15], 0);
    }
}
