//! Memory-mapped file access.
//!
//! Ported from:
//! - lib/marisa/grimoire/io/mapper.h
//! - lib/marisa/grimoire/io/mapper.cc
//!
//! Mapper provides read-only access to data through memory mapping.
//! This implementation supports both file-backed memory mapping and borrowed memory.

use memmap2::Mmap;
use std::fs::File;
use std::io;

/// Mapper for memory-mapped data access.
///
/// Mapper provides read-only access to data, primarily used for
/// deserializing trie structures from memory or files.
///
/// The mapper can work in two modes:
/// - File-backed memory mapping using `memmap2::Mmap`
/// - Borrowed memory slices (for testing or in-memory data)
pub struct Mapper {
    /// File-backed memory map.
    mmap: Option<Mmap>,
    /// Borrowed memory reference (static lifetime for safety).
    borrowed: Option<&'static [u8]>,
    /// Current read position.
    position: usize,
}

impl Default for Mapper {
    fn default() -> Self {
        Self::new()
    }
}

impl Mapper {
    /// Creates a new empty mapper.
    pub fn new() -> Self {
        Mapper {
            mmap: None,
            borrowed: None,
            position: 0,
        }
    }

    /// Opens a mapper from a file using memory mapping.
    ///
    /// # Arguments
    ///
    /// * `filename` - Path to the file to map
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or mapped.
    ///
    /// # Safety
    ///
    /// This function uses memory mapping which is inherently unsafe because:
    /// - The file could be modified externally while mapped
    /// - The file could be truncated while mapped
    ///
    /// The caller must ensure that the file is not modified during the lifetime
    /// of the Mapper. Files should be opened read-only.
    pub fn open_file(filename: &str) -> io::Result<Self> {
        let file = File::open(filename)?;
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Mapper {
            mmap: Some(mmap),
            borrowed: None,
            position: 0,
        })
    }

    /// Opens a mapper from a byte slice.
    ///
    /// # Arguments
    ///
    /// * `data` - Byte slice to map (must have static lifetime)
    ///
    /// # Safety
    ///
    /// The data must have a valid static lifetime to ensure it outlives
    /// the mapper and any structures that reference it.
    pub fn open_memory(data: &'static [u8]) -> Self {
        Mapper {
            mmap: None,
            borrowed: Some(data),
            position: 0,
        }
    }

    /// Opens a mapper from a byte slice (legacy method).
    ///
    /// # Arguments
    ///
    /// * `data` - Byte slice to map (must have static lifetime)
    ///
    /// # Note
    ///
    /// This method is provided for compatibility. New code should use
    /// `open_memory()` instead.
    pub fn open(data: &'static [u8]) -> Self {
        Self::open_memory(data)
    }

    /// Returns a reference to the underlying data.
    ///
    /// Returns the data from either the memory map or borrowed slice.
    fn data(&self) -> &[u8] {
        self.mmap
            .as_ref()
            .map(|m| &m[..])
            .or(self.borrowed)
            .unwrap_or(&[])
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
        let data = self.data();
        if data.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Mapper not open",
            ));
        }

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

    /// Maps and returns a single value of type T from the current position.
    ///
    /// Convenience method that returns the value instead of taking a mutable reference.
    ///
    /// # Errors
    ///
    /// Returns an error if the mapper is not open or if there's insufficient data.
    ///
    /// # Safety
    ///
    /// This function reads raw bytes into the memory representation of T.
    /// The caller must ensure T is safe to initialize from arbitrary bytes.
    pub fn map_value<T: Copy + Default>(&mut self) -> io::Result<T> {
        let mut value = T::default();
        self.map(&mut value)?;
        Ok(value)
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
    /// This function reads raw bytes into the memory representation of `T`.
    /// The caller must ensure T is safe to initialize from arbitrary bytes.
    pub fn map_slice<T: Copy>(&mut self, values: &mut [T]) -> io::Result<()> {
        if values.is_empty() {
            return Ok(());
        }

        let data = self.data();
        if data.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Mapper not open",
            ));
        }

        let size = std::mem::size_of_val(values);
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
        let data = self.data();
        if data.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Mapper not open",
            ));
        }

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
        self.mmap.is_some() || self.borrowed.is_some()
    }

    /// Returns the current position.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Returns the total size of mapped data.
    pub fn size(&self) -> usize {
        self.data().len()
    }

    /// Closes the mapper.
    pub fn clear(&mut self) {
        self.mmap = None;
        self.borrowed = None;
        self.position = 0;
    }

    /// Swaps with another mapper.
    pub fn swap(&mut self, other: &mut Mapper) {
        std::mem::swap(&mut self.mmap, &mut other.mmap);
        std::mem::swap(&mut self.borrowed, &mut other.borrowed);
        std::mem::swap(&mut self.position, &mut other.position);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_mapper_new() {
        let mapper = Mapper::new();
        assert!(!mapper.is_open());
        assert_eq!(mapper.position(), 0);
    }

    #[test]
    fn test_mapper_open_memory() {
        static DATA: [u8; 5] = [1, 2, 3, 4, 5];
        let mapper = Mapper::open_memory(&DATA);
        assert!(mapper.is_open());
        assert_eq!(mapper.size(), 5);
        assert_eq!(mapper.position(), 0);
    }

    #[test]
    fn test_mapper_open_file() {
        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = vec![1u8, 2, 3, 4, 5];
        temp_file.write_all(&test_data).unwrap();
        temp_file.flush().unwrap();

        let path = temp_file.path().to_str().unwrap();
        let mapper = Mapper::open_file(path).unwrap();

        assert!(mapper.is_open());
        assert_eq!(mapper.size(), 5);
        assert_eq!(mapper.position(), 0);
    }

    #[test]
    fn test_mapper_open_file_not_found() {
        let result = Mapper::open_file("/nonexistent/file.dat");
        assert!(result.is_err());
    }

    #[test]
    fn test_mapper_map_u32() {
        static DATA: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
        let mut mapper = Mapper::open_memory(&DATA);

        let mut value: u32 = 0;
        mapper.map(&mut value).unwrap();

        // Little-endian: 0x04030201
        assert_eq!(value, 0x04030201);
        assert_eq!(mapper.position(), 4);
    }

    #[test]
    fn test_mapper_map_value() {
        static DATA: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
        let mut mapper = Mapper::open_memory(&DATA);

        let value: u32 = mapper.map_value().unwrap();

        // Little-endian: 0x04030201
        assert_eq!(value, 0x04030201);
        assert_eq!(mapper.position(), 4);
    }

    #[test]
    fn test_mapper_map_slice() {
        static DATA: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let mut mapper = Mapper::open_memory(&DATA);

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
        static DATA: [u8; 4] = [1, 2, 3, 4];
        let mut mapper = Mapper::open_memory(&DATA);

        let mut values: [u8; 0] = [];
        mapper.map_slice(&mut values).unwrap();

        assert_eq!(mapper.position(), 0);
    }

    #[test]
    fn test_mapper_seek() {
        static DATA: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let mut mapper = Mapper::open_memory(&DATA);

        mapper.seek(4).unwrap();
        assert_eq!(mapper.position(), 4);

        let mut value: u8 = 0;
        mapper.map(&mut value).unwrap();
        assert_eq!(value, 5);
    }

    #[test]
    fn test_mapper_seek_zero() {
        static DATA: [u8; 4] = [1, 2, 3, 4];
        let mut mapper = Mapper::open_memory(&DATA);

        mapper.seek(0).unwrap();
        assert_eq!(mapper.position(), 0);
    }

    #[test]
    fn test_mapper_clear() {
        static DATA: [u8; 4] = [1, 2, 3, 4];
        let mut mapper = Mapper::open_memory(&DATA);

        assert!(mapper.is_open());
        mapper.clear();
        assert!(!mapper.is_open());
        assert_eq!(mapper.position(), 0);
    }

    #[test]
    fn test_mapper_swap() {
        static DATA1: [u8; 3] = [1, 2, 3];
        static DATA2: [u8; 5] = [4, 5, 6, 7, 8];

        let mut mapper1 = Mapper::open_memory(&DATA1);
        let mut mapper2 = Mapper::open_memory(&DATA2);

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
        static DATA: [u8; 2] = [1, 2];
        let mut mapper = Mapper::open_memory(&DATA);

        let mut value: u32 = 0;
        let result = mapper.map(&mut value);
        assert!(result.is_err());
    }

    #[test]
    fn test_mapper_seek_past_end() {
        static DATA: [u8; 4] = [1, 2, 3, 4];
        let mut mapper = Mapper::open_memory(&DATA);

        let result = mapper.seek(10);
        assert!(result.is_err());
    }

    #[test]
    fn test_mapper_multiple_maps() {
        // Create static data with u32 and u64 values
        static DATA: [u8; 12] = [
            42, 0, 0, 0, // 42 as u32 (little-endian)
            100, 0, 0, 0, 0, 0, 0, 0, // 100 as u64 (little-endian)
        ];

        let mut mapper = Mapper::open_memory(&DATA);

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

    #[test]
    fn test_mapper_file_mapping() {
        // Create a temporary file with some data
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = vec![0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        temp_file.write_all(&test_data).unwrap();
        temp_file.flush().unwrap();

        let path = temp_file.path().to_str().unwrap();
        let mut mapper = Mapper::open_file(path).unwrap();

        // Test reading from file-backed mmap
        let mut values = [0u8; 4];
        mapper.map_slice(&mut values).unwrap();
        assert_eq!(values, [1, 2, 3, 4]);

        mapper.map_slice(&mut values).unwrap();
        assert_eq!(values, [5, 6, 7, 8]);
    }
}
