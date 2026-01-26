//! Space-efficient integer vector.
//!
//! Ported from: lib/marisa/grimoire/vector/flat-vector.h
//!
//! This module provides FlatVector, which stores integers using the minimum
//! number of bits required based on the maximum value. For example, if all
//! values fit in 3 bits, it uses 3 bits per value instead of 32.

use super::vector::Vector;
use crate::base::WORD_SIZE;

#[cfg(target_pointer_width = "64")]
type Unit = u64;
#[cfg(target_pointer_width = "32")]
type Unit = u32;

/// Flat vector for space-efficient integer storage.
///
/// FlatVector stores unsigned 32-bit integers using bit-packing to save space.
/// It calculates the minimum number of bits needed based on the maximum value
/// and packs all values using that bit-width.
#[derive(Default)]
pub struct FlatVector {
    /// Storage for bit-packed values.
    units: Vector<Unit>,
    /// Number of bits per value.
    value_size: usize,
    /// Mask for extracting a value (all 1s for value_size bits).
    mask: u32,
    /// Number of values stored.
    size: usize,
}

impl FlatVector {
    /// Creates a new empty flat vector.
    #[inline]
    pub fn new() -> Self {
        FlatVector {
            units: Vector::new(),
            value_size: 0,
            mask: 0,
            size: 0,
        }
    }

    /// Builds the flat vector from a vector of values.
    ///
    /// This determines the optimal bit-width based on the maximum value
    /// and packs all values accordingly.
    ///
    /// # Arguments
    ///
    /// * `values` - Vector of u32 values to store
    pub fn build(&mut self, values: &Vector<u32>) {
        let mut temp = FlatVector::new();
        temp.build_internal(values);
        self.swap(&mut temp);
    }

    /// Returns the value at the given index.
    ///
    /// # Arguments
    ///
    /// * `i` - Index of the value to retrieve
    ///
    /// # Returns
    ///
    /// The value at index i
    ///
    /// # Panics
    ///
    /// Panics if i >= size()
    pub fn get(&self, i: usize) -> u32 {
        assert!(i < self.size, "Index out of bounds");

        let pos = i * self.value_size;
        let unit_id = pos / WORD_SIZE;
        let unit_offset = pos % WORD_SIZE;

        if (unit_offset + self.value_size) <= WORD_SIZE {
            // Value fits in a single unit
            ((self.units[unit_id] >> unit_offset) as u32) & self.mask
        } else {
            // Value spans two units
            let low_bits = (self.units[unit_id] >> unit_offset) as u32;
            let high_bits = (self.units[unit_id + 1] << (WORD_SIZE - unit_offset)) as u32;
            (low_bits | high_bits) & self.mask
        }
    }

    /// Returns the number of bits per value.
    #[inline]
    pub fn value_size(&self) -> usize {
        self.value_size
    }

    /// Returns the mask used for extracting values.
    #[inline]
    pub fn mask(&self) -> u32 {
        self.mask
    }

    /// Returns true if the vector is empty.
    #[inline]
    pub fn empty(&self) -> bool {
        self.size == 0
    }

    /// Returns the number of values stored.
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the total size in bytes.
    #[inline]
    pub fn total_size(&self) -> usize {
        self.units.total_size()
    }

    /// Returns the I/O size needed for serialization.
    #[inline]
    pub fn io_size(&self) -> usize {
        self.units.io_size() + std::mem::size_of::<u32>() * 2 + std::mem::size_of::<u64>()
    }

    /// Clears the flat vector.
    #[inline]
    pub fn clear(&mut self) {
        *self = FlatVector::new();
    }

    /// Swaps the contents of two flat vectors.
    pub fn swap(&mut self, other: &mut FlatVector) {
        self.units.swap(&mut other.units);
        std::mem::swap(&mut self.value_size, &mut other.value_size);
        std::mem::swap(&mut self.mask, &mut other.mask);
        std::mem::swap(&mut self.size, &mut other.size);
    }

    /// Maps the flat vector from a mapper.
    ///
    /// Format (matching C++ marisa-trie):
    /// - units: `Vector<u64>`
    /// - value_size: u32 (must be <= 32)
    /// - mask: u32
    /// - size: u64
    ///
    /// # Arguments
    ///
    /// * `mapper` - Mapper to read from
    ///
    /// # Errors
    ///
    /// Returns an error if mapping fails or if value_size > 32.
    pub fn map(&mut self, mapper: &mut crate::grimoire::io::Mapper) -> std::io::Result<()> {
        // Map units
        self.units.map(mapper)?;

        // Map value_size and validate
        let temp_value_size: u32 = mapper.map_value()?;
        if temp_value_size > 32 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "value_size exceeds 32",
            ));
        }
        self.value_size = temp_value_size as usize;

        // Map mask
        let temp_mask: u32 = mapper.map_value()?;
        self.mask = temp_mask;

        // Map size
        let temp_size: u64 = mapper.map_value()?;
        self.size = temp_size as usize;

        Ok(())
    }

    /// Reads the flat vector from a reader.
    ///
    /// Format (matching C++ marisa-trie):
    /// - units: `Vector<u64>`
    /// - value_size: u32 (must be <= 32)
    /// - mask: u32
    /// - size: u64
    ///
    /// # Arguments
    ///
    /// * `reader` - Reader to read from
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails or if value_size > 32.
    pub fn read(&mut self, reader: &mut crate::grimoire::io::Reader) -> std::io::Result<()> {
        // Read units
        self.units.read(reader)?;

        // Read value_size and validate
        let temp_value_size: u32 = reader.read()?;
        if temp_value_size > 32 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "value_size exceeds 32",
            ));
        }
        self.value_size = temp_value_size as usize;

        // Read mask
        let temp_mask: u32 = reader.read()?;
        self.mask = temp_mask;

        // Read size
        let temp_size: u64 = reader.read()?;
        self.size = temp_size as usize;

        Ok(())
    }

    /// Writes the flat vector to a writer.
    ///
    /// Format (matching C++ marisa-trie):
    /// - units: `Vector<u64>`
    /// - value_size: u32
    /// - mask: u32
    /// - size: u64
    ///
    /// # Arguments
    ///
    /// * `writer` - Writer to write to
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn write(&self, writer: &mut crate::grimoire::io::Writer) -> std::io::Result<()> {
        // Write units
        self.units.write(writer)?;

        // Write value_size, mask, size
        writer.write(&(self.value_size as u32))?;
        writer.write(&self.mask)?;
        writer.write(&(self.size as u64))?;

        Ok(())
    }

    /// Internal build implementation.
    fn build_internal(&mut self, values: &Vector<u32>) {
        // Find maximum value to determine bit-width needed
        let mut max_value = 0u32;
        for i in 0..values.size() {
            if values[i] > max_value {
                max_value = values[i];
            }
        }

        // Calculate number of bits needed for max_value
        let mut value_size = 0usize;
        let mut temp_max = max_value;
        while temp_max != 0 {
            value_size += 1;
            temp_max >>= 1;
        }

        // Calculate number of units needed
        // Align to 64-bit boundaries (2 units on 32-bit, 1 unit on 64-bit)
        let num_units = if values.empty() {
            0
        } else if value_size == 0 {
            64 / WORD_SIZE
        } else {
            let bits_needed = value_size as u64 * values.size() as u64;
            let mut num_units =
                ((bits_needed + (WORD_SIZE as u64 - 1)) / WORD_SIZE as u64) as usize;
            // Round up to 64-bit alignment
            let alignment = 64 / WORD_SIZE;
            num_units += num_units % alignment;
            num_units
        };

        self.units.resize(num_units, 0);
        if num_units > 0 {
            // Initialize last unit to 0
            self.units[num_units - 1] = 0;
        }

        self.value_size = value_size;
        self.mask = if value_size != 0 {
            u32::MAX >> (32 - value_size)
        } else {
            0
        };
        self.size = values.size();

        // Set all values
        for i in 0..values.size() {
            self.set(i, values[i]);
        }
    }

    /// Sets the value at the given index.
    ///
    /// # Arguments
    ///
    /// * `i` - Index to set
    /// * `value` - Value to store
    ///
    /// # Panics
    ///
    /// Panics if i >= size() or if value > mask
    fn set(&mut self, i: usize, value: u32) {
        assert!(i < self.size, "Index out of bounds");
        assert!(value <= self.mask, "Value exceeds maximum");

        let pos = i * self.value_size;
        let unit_id = pos / WORD_SIZE;
        let unit_offset = pos % WORD_SIZE;

        // Clear the bits for this value and set new value
        self.units[unit_id] &= !(Unit::from(self.mask) << unit_offset);
        self.units[unit_id] |= Unit::from(value & self.mask) << unit_offset;

        // Handle case where value spans two units
        if (unit_offset + self.value_size) > WORD_SIZE {
            let high_shift = WORD_SIZE - unit_offset;
            self.units[unit_id + 1] &= !(Unit::from(self.mask) >> high_shift);
            self.units[unit_id + 1] |= Unit::from(value & self.mask) >> high_shift;
        }
    }

    // TODO: Implement map(), read(), write() for serialization
}

// Note: We cannot implement Index<usize> for FlatVector because
// Index::index() must return a reference, but we need to return
// a u32 value. Use get() method instead.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_vector_new() {
        let fv = FlatVector::new();
        assert_eq!(fv.size(), 0);
        assert!(fv.empty());
        assert_eq!(fv.value_size(), 0);
        assert_eq!(fv.mask(), 0);
    }

    #[test]
    fn test_flat_vector_build_small() {
        let mut values = Vector::new();
        values.push_back(0);
        values.push_back(1);
        values.push_back(2);
        values.push_back(3);

        let mut fv = FlatVector::new();
        fv.build(&values);

        assert_eq!(fv.size(), 4);
        assert_eq!(fv.value_size(), 2); // 2 bits needed for value 3
        assert_eq!(fv.mask(), 0b11);

        assert_eq!(fv.get(0), 0);
        assert_eq!(fv.get(1), 1);
        assert_eq!(fv.get(2), 2);
        assert_eq!(fv.get(3), 3);
    }

    #[test]
    fn test_flat_vector_build_powers_of_two() {
        let mut values = Vector::new();
        values.push_back(1);
        values.push_back(2);
        values.push_back(4);
        values.push_back(8);
        values.push_back(16);

        let mut fv = FlatVector::new();
        fv.build(&values);

        assert_eq!(fv.size(), 5);
        assert_eq!(fv.value_size(), 5); // 5 bits needed for value 16

        assert_eq!(fv.get(0), 1);
        assert_eq!(fv.get(1), 2);
        assert_eq!(fv.get(2), 4);
        assert_eq!(fv.get(3), 8);
        assert_eq!(fv.get(4), 16);
    }

    #[test]
    fn test_flat_vector_all_zeros() {
        let mut values = Vector::new();
        for _ in 0..10 {
            values.push_back(0);
        }

        let mut fv = FlatVector::new();
        fv.build(&values);

        assert_eq!(fv.size(), 10);
        assert_eq!(fv.value_size(), 0); // 0 bits needed for all zeros

        for i in 0..10 {
            assert_eq!(fv.get(i), 0);
        }
    }

    #[test]
    fn test_flat_vector_large_values() {
        let mut values = Vector::new();
        values.push_back(255);
        values.push_back(256);
        values.push_back(1000);
        values.push_back(65535);

        let mut fv = FlatVector::new();
        fv.build(&values);

        assert_eq!(fv.size(), 4);
        assert_eq!(fv.value_size(), 16); // 16 bits needed for 65535

        assert_eq!(fv.get(0), 255);
        assert_eq!(fv.get(1), 256);
        assert_eq!(fv.get(2), 1000);
        assert_eq!(fv.get(3), 65535);
    }

    #[test]
    fn test_flat_vector_many_values() {
        let mut values = Vector::new();
        for i in 0..100 {
            values.push_back(i % 16); // Values 0-15
        }

        let mut fv = FlatVector::new();
        fv.build(&values);

        assert_eq!(fv.size(), 100);
        assert_eq!(fv.value_size(), 4); // 4 bits needed for value 15

        for i in 0..100 {
            assert_eq!(fv.get(i), (i % 16) as u32);
        }
    }

    #[test]
    fn test_flat_vector_clear() {
        let mut values = Vector::new();
        values.push_back(1);
        values.push_back(2);

        let mut fv = FlatVector::new();
        fv.build(&values);

        assert_eq!(fv.size(), 2);
        fv.clear();
        assert_eq!(fv.size(), 0);
        assert!(fv.empty());
    }

    #[test]
    fn test_flat_vector_swap() {
        let mut values1 = Vector::new();
        values1.push_back(1);
        values1.push_back(2);

        let mut values2 = Vector::new();
        values2.push_back(10);
        values2.push_back(20);
        values2.push_back(30);

        let mut fv1 = FlatVector::new();
        let mut fv2 = FlatVector::new();

        fv1.build(&values1);
        fv2.build(&values2);

        fv1.swap(&mut fv2);

        assert_eq!(fv1.size(), 3);
        assert_eq!(fv2.size(), 2);
        assert_eq!(fv1.get(0), 10);
        assert_eq!(fv2.get(0), 1);
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_flat_vector_out_of_bounds() {
        let mut values = Vector::new();
        values.push_back(1);

        let mut fv = FlatVector::new();
        fv.build(&values);

        fv.get(1); // Should panic
    }

    #[test]
    fn test_flat_vector_empty() {
        let values = Vector::new();
        let mut fv = FlatVector::new();
        fv.build(&values);

        assert_eq!(fv.size(), 0);
        assert!(fv.empty());
    }

    #[test]
    fn test_flat_vector_write_read() {
        // Rust-specific: Test FlatVector serialization
        use crate::grimoire::io::{Reader, Writer};

        let mut values = Vector::new();
        for i in 0..100u32 {
            values.push_back((i * 7) % 256); // Values 0-255 need 8 bits
        }

        let mut fv = FlatVector::new();
        fv.build(&values);

        // Write to buffer
        let mut writer = Writer::from_vec(Vec::new());
        fv.write(&mut writer).unwrap();

        let data = writer.into_inner().unwrap();

        // Read back
        let mut reader = Reader::from_bytes(&data);
        let mut fv2 = FlatVector::new();
        fv2.read(&mut reader).unwrap();

        // Verify
        assert_eq!(fv2.size(), 100);
        for i in 0..100usize {
            assert_eq!(fv2.get(i), ((i as u32 * 7) % 256));
        }
    }

    #[test]
    fn test_flat_vector_write_read_empty() {
        // Rust-specific: Test empty FlatVector serialization
        use crate::grimoire::io::{Reader, Writer};

        let fv = FlatVector::new();

        // Write to buffer
        let mut writer = Writer::from_vec(Vec::new());
        fv.write(&mut writer).unwrap();

        let data = writer.into_inner().unwrap();

        // Read back
        let mut reader = Reader::from_bytes(&data);
        let mut fv2 = FlatVector::new();
        fv2.read(&mut reader).unwrap();

        assert_eq!(fv2.size(), 0);
        assert!(fv2.empty());
    }

    #[test]
    fn test_flat_vector_read_invalid_value_size() {
        // Rust-specific: Test validation of value_size <= 32
        use crate::grimoire::io::{Reader, Writer};

        // Create invalid data where value_size > 32
        let mut writer = Writer::from_vec(Vec::new());

        // Write empty units vector
        let empty_vec: crate::grimoire::vector::vector::Vector<u64> =
            crate::grimoire::vector::vector::Vector::new();
        empty_vec.write(&mut writer).unwrap();

        // Write value_size = 40 (invalid!), mask = 0, size = 0
        writer.write(&40u32).unwrap();
        writer.write(&0u32).unwrap();
        writer.write(&0u64).unwrap();

        let data = writer.into_inner().unwrap();

        // Try to read - should fail
        let mut reader = Reader::from_bytes(&data);
        let mut fv = FlatVector::new();
        let result = fv.read(&mut reader);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }
}
