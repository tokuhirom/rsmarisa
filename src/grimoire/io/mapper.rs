//! Memory-mapped file access.
//!
//! Ported from:
//! - lib/marisa/grimoire/io/mapper.h
//! - lib/marisa/grimoire/io/mapper.cc
//!
//! Mapper provides read-only access to data through memory mapping.
//! This is a simplified implementation that works with byte slices.

use std::io;

/// Mapper for memory-mapped data access.
///
/// Mapper provides read-only access to data, primarily used for
/// deserializing trie structures from memory or files.
pub struct Mapper<'a> {
    /// Data buffer being mapped.
    data: Option<&'a [u8]>,
    /// Current read position.
    position: usize,
}

impl<'a> Default for Mapper<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Mapper<'a> {
    /// Creates a new empty mapper.
    pub fn new() -> Self {
        Mapper {
            data: None,
            position: 0,
        }
    }

    /// Opens a mapper from a byte slice.
    ///
    /// # Arguments
    ///
    /// * `data` - Byte slice to map
    pub fn open(data: &'a [u8]) -> Self {
        Mapper {
            data: Some(data),
            position: 0,
        }
    }

    /// Maps a single value of type T from the current position.
    ///
    /// # Arguments
    ///
    /// * `value` - Mutable reference to store the mapped value
    ///
    /// # Errors
    ///
    /// Returns an error if the mapper is not open or if there's insufficient data.
    ///
    /// # Safety
    ///
    /// This function reads raw bytes into the memory representation of T.
    /// The caller must ensure T is safe to initialize from arbitrary bytes.
    pub fn map<T: Copy>(&mut self, value: &mut T) -> io::Result<()> {
        let data = self
            .data
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "Mapper not open"))?;

        let size = std::mem::size_of::<T>();
        if self.position + size > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Insufficient data to map",
            ));
        }

        let slice = &data[self.position..self.position + size];
        unsafe {
            std::ptr::copy_nonoverlapping(slice.as_ptr(), value as *mut T as *mut u8, size);
        }

        self.position += size;
        Ok(())
    }

    /// Maps a slice of values from the current position.
    ///
    /// # Arguments
    ///
    /// * `values` - Mutable slice to fill with mapped values
    ///
    /// # Errors
    ///
    /// Returns an error if the mapper is not open or if there's insufficient data.
    ///
    /// # Safety
    ///
    /// This function reads raw bytes into the memory representation of [T].
    /// The caller must ensure T is safe to initialize from arbitrary bytes.
    pub fn map_slice<T: Copy>(&mut self, values: &mut [T]) -> io::Result<()> {
        if values.is_empty() {
            return Ok(());
        }

        let data = self
            .data
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "Mapper not open"))?;

        let size = std::mem::size_of::<T>() * values.len();
        if self.position + size > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Insufficient data to map",
            ));
        }

        let slice = &data[self.position..self.position + size];
        unsafe {
            std::ptr::copy_nonoverlapping(slice.as_ptr(), values.as_mut_ptr() as *mut u8, size);
        }

        self.position += size;
        Ok(())
    }

    /// Seeks forward by the specified number of bytes.
    ///
    /// # Arguments
    ///
    /// * `size` - Number of bytes to skip
    ///
    /// # Errors
    ///
    /// Returns an error if the mapper is not open or if seeking past the end.
    pub fn seek(&mut self, size: usize) -> io::Result<()> {
        let data = self
            .data
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "Mapper not open"))?;

        if self.position + size > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Seek past end of data",
            ));
        }

        self.position += size;
        Ok(())
    }

    /// Checks if the mapper is open.
    pub fn is_open(&self) -> bool {
        self.data.is_some()
    }

    /// Returns the current position.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Returns the total size of mapped data.
    pub fn size(&self) -> usize {
        self.data.map(|d| d.len()).unwrap_or(0)
    }

    /// Closes the mapper.
    pub fn clear(&mut self) {
        self.data = None;
        self.position = 0;
    }

    /// Swaps with another mapper.
    pub fn swap(&mut self, other: &mut Mapper<'a>) {
        std::mem::swap(&mut self.data, &mut other.data);
        std::mem::swap(&mut self.position, &mut other.position);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapper_new() {
        let mapper = Mapper::new();
        assert!(!mapper.is_open());
        assert_eq!(mapper.position(), 0);
    }

    #[test]
    fn test_mapper_open() {
        let data = vec![1u8, 2, 3, 4, 5];
        let mapper = Mapper::open(&data);
        assert!(mapper.is_open());
        assert_eq!(mapper.size(), 5);
        assert_eq!(mapper.position(), 0);
    }

    #[test]
    fn test_mapper_map_u32() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut mapper = Mapper::open(&data);

        let mut value: u32 = 0;
        mapper.map(&mut value).unwrap();

        // Little-endian: 0x04030201
        assert_eq!(value, 0x04030201);
        assert_eq!(mapper.position(), 4);
    }

    #[test]
    fn test_mapper_map_slice() {
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let mut mapper = Mapper::open(&data);

        let mut values = [0u8; 4];
        mapper.map_slice(&mut values).unwrap();

        assert_eq!(values, [1, 2, 3, 4]);
        assert_eq!(mapper.position(), 4);

        // Map next 4 bytes
        mapper.map_slice(&mut values).unwrap();
        assert_eq!(values, [5, 6, 7, 8]);
        assert_eq!(mapper.position(), 8);
    }

    #[test]
    fn test_mapper_map_empty_slice() {
        let data = vec![1u8, 2, 3, 4];
        let mut mapper = Mapper::open(&data);

        let mut values: [u8; 0] = [];
        mapper.map_slice(&mut values).unwrap();

        assert_eq!(mapper.position(), 0);
    }

    #[test]
    fn test_mapper_seek() {
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let mut mapper = Mapper::open(&data);

        mapper.seek(4).unwrap();
        assert_eq!(mapper.position(), 4);

        let mut value: u8 = 0;
        mapper.map(&mut value).unwrap();
        assert_eq!(value, 5);
    }

    #[test]
    fn test_mapper_seek_zero() {
        let data = vec![1u8, 2, 3, 4];
        let mut mapper = Mapper::open(&data);

        mapper.seek(0).unwrap();
        assert_eq!(mapper.position(), 0);
    }

    #[test]
    fn test_mapper_clear() {
        let data = vec![1u8, 2, 3, 4];
        let mut mapper = Mapper::open(&data);

        assert!(mapper.is_open());
        mapper.clear();
        assert!(!mapper.is_open());
        assert_eq!(mapper.position(), 0);
    }

    #[test]
    fn test_mapper_swap() {
        let data1 = vec![1u8, 2, 3];
        let data2 = vec![4u8, 5, 6, 7, 8];

        let mut mapper1 = Mapper::open(&data1);
        let mut mapper2 = Mapper::open(&data2);

        mapper1.seek(1).unwrap();
        mapper2.seek(2).unwrap();

        assert_eq!(mapper1.size(), 3);
        assert_eq!(mapper2.size(), 5);
        assert_eq!(mapper1.position(), 1);
        assert_eq!(mapper2.position(), 2);

        mapper1.swap(&mut mapper2);

        assert_eq!(mapper1.size(), 5);
        assert_eq!(mapper2.size(), 3);
        assert_eq!(mapper1.position(), 2);
        assert_eq!(mapper2.position(), 1);
    }

    #[test]
    fn test_mapper_not_open() {
        let mut mapper = Mapper::new();
        let mut value: u32 = 0;

        let result = mapper.map(&mut value);
        assert!(result.is_err());
    }

    #[test]
    fn test_mapper_insufficient_data() {
        let data = vec![1u8, 2];
        let mut mapper = Mapper::open(&data);

        let mut value: u32 = 0;
        let result = mapper.map(&mut value);
        assert!(result.is_err());
    }

    #[test]
    fn test_mapper_seek_past_end() {
        let data = vec![1u8, 2, 3, 4];
        let mut mapper = Mapper::open(&data);

        let result = mapper.seek(10);
        assert!(result.is_err());
    }

    #[test]
    fn test_mapper_multiple_maps() {
        let mut data = Vec::new();
        data.extend_from_slice(&42u32.to_le_bytes());
        data.extend_from_slice(&100u64.to_le_bytes());

        let mut mapper = Mapper::open(&data);

        let mut val_u32: u32 = 0;
        mapper.map(&mut val_u32).unwrap();
        assert_eq!(val_u32, 42);

        let mut val_u64: u64 = 0;
        mapper.map(&mut val_u64).unwrap();
        assert_eq!(val_u64, 100);
    }

    #[test]
    fn test_mapper_default() {
        let mapper = Mapper::default();
        assert!(!mapper.is_open());
    }
}
