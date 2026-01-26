//! Writer for serializing trie data.
//!
//! Ported from:
//! - lib/marisa/grimoire/io/writer.h
//! - lib/marisa/grimoire/io/writer.cc
//!
//! Writer provides methods to write binary data to various destinations
//! including files, byte vectors, and any type implementing std::io::Write.

use std::fs::File;
use std::io::{self, Write as IoWrite};
use std::path::Path;

/// Writer for writing binary data to various destinations.
///
/// Writer wraps a std::io::Write implementation and provides convenient
/// methods for writing typed data and seeking forward with zero padding.
pub struct Writer {
    /// The underlying writer, boxed for trait object support.
    writer: Option<Box<dyn IoWrite>>,
    /// Optional buffer for in-memory writing (for testing).
    buffer: Option<Vec<u8>>,
}

impl Writer {
    /// Creates a new empty writer.
    pub fn new() -> Self {
        Writer {
            writer: None,
            buffer: None,
        }
    }

    /// Opens a file for writing.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to create/overwrite
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Writer {
            writer: Some(Box::new(file)),
            buffer: None,
        })
    }

    /// Creates a writer from any type implementing std::io::Write.
    ///
    /// # Arguments
    ///
    /// * `writer` - Any type implementing Write
    pub fn from_writer<W: IoWrite + 'static>(writer: W) -> Self {
        Writer {
            writer: Some(Box::new(writer)),
            buffer: None,
        }
    }

    /// Creates a writer that writes to a `Vec<u8>`.
    pub fn from_vec(vec: Vec<u8>) -> Self {
        Writer {
            writer: None,
            buffer: Some(vec),
        }
    }

    /// Writes a single value of type T.
    ///
    /// # Arguments
    ///
    /// * `value` - Reference to the value to write
    ///
    /// # Errors
    ///
    /// Returns an error if the writer is not open or if writing fails.
    ///
    /// # Safety
    ///
    /// This function writes the raw bytes of T's memory representation.
    /// It's safe for types like u32, u64, but the caller must ensure T
    /// has a stable binary representation.
    pub fn write<T>(&mut self, value: &T) -> io::Result<()> {
        let size = std::mem::size_of::<T>();
        let slice = unsafe { std::slice::from_raw_parts(value as *const T as *const u8, size) };

        if let Some(buffer) = &mut self.buffer {
            buffer.extend_from_slice(slice);
            Ok(())
        } else if let Some(writer) = &mut self.writer {
            writer.write_all(slice)?;
            writer.flush()?;
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Writer not open",
            ))
        }
    }

    /// Writes multiple values from a slice.
    ///
    /// # Arguments
    ///
    /// * `values` - Slice of values to write
    ///
    /// # Errors
    ///
    /// Returns an error if the writer is not open or if writing fails.
    ///
    /// # Safety
    ///
    /// This function writes the raw bytes of `T`'s memory representation.
    /// The caller must ensure T has a stable binary representation.
    pub fn write_slice<T>(&mut self, values: &[T]) -> io::Result<()> {
        if values.is_empty() {
            return Ok(());
        }

        let size = std::mem::size_of_val(values);
        let slice = unsafe { std::slice::from_raw_parts(values.as_ptr() as *const u8, size) };

        if let Some(buffer) = &mut self.buffer {
            buffer.extend_from_slice(slice);
            Ok(())
        } else if let Some(writer) = &mut self.writer {
            writer.write_all(slice)?;
            writer.flush()?;
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Writer not open",
            ))
        }
    }

    /// Seeks forward by writing the specified number of zero bytes.
    ///
    /// This method writes zero padding. For small sizes (<= 16 bytes),
    /// it uses a stack buffer. For larger sizes, it uses a 1024-byte buffer.
    ///
    /// # Arguments
    ///
    /// * `size` - Number of zero bytes to write
    ///
    /// # Errors
    ///
    /// Returns an error if the writer is not open or if writing fails.
    pub fn seek(&mut self, size: usize) -> io::Result<()> {
        if size == 0 {
            return Ok(());
        }

        if let Some(buffer) = &mut self.buffer {
            buffer.resize(buffer.len() + size, 0);
            Ok(())
        } else if let Some(writer) = &mut self.writer {
            if size <= 16 {
                let buf = [0u8; 16];
                writer.write_all(&buf[..size])?;
            } else {
                let buf = [0u8; 1024];
                let mut remaining = size;
                while remaining > 0 {
                    let count = remaining.min(buf.len());
                    writer.write_all(&buf[..count])?;
                    remaining -= count;
                }
            }
            writer.flush()?;
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Writer not open",
            ))
        }
    }

    /// Checks if the writer is open.
    pub fn is_open(&self) -> bool {
        self.writer.is_some() || self.buffer.is_some()
    }

    /// Closes the writer and releases resources.
    pub fn clear(&mut self) {
        self.writer = None;
        self.buffer = None;
    }

    /// Extracts the inner `Vec<u8>` if the writer was created with from_vec.
    ///
    /// # Errors
    ///
    /// Returns an error if the writer doesn't have a buffer.
    pub fn into_inner(self) -> io::Result<Vec<u8>> {
        self.buffer.ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "Writer does not have a buffer")
        })
    }
}

impl Default for Writer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer_new() {
        let writer = Writer::new();
        assert!(!writer.is_open());
    }

    #[test]
    fn test_writer_from_vec() {
        let writer = Writer::from_vec(Vec::new());
        assert!(writer.is_open());
    }

    #[test]
    fn test_writer_write_u32() {
        let mut writer = Writer::from_vec(Vec::new());

        let value: u32 = 0x04030201;
        writer.write(&value).unwrap();

        let data = writer.into_inner().unwrap();
        assert_eq!(data, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_writer_write_slice() {
        let mut writer = Writer::from_vec(Vec::new());

        let values = [1u8, 2, 3, 4];
        writer.write_slice(&values).unwrap();

        let data = writer.into_inner().unwrap();
        assert_eq!(data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_writer_write_empty_slice() {
        let mut writer = Writer::from_vec(Vec::new());

        let values: [u8; 0] = [];
        writer.write_slice(&values).unwrap();

        let data = writer.into_inner().unwrap();
        assert_eq!(data.len(), 0);
    }

    #[test]
    fn test_writer_seek() {
        let mut writer = Writer::from_vec(Vec::new());

        writer.write(&42u8).unwrap();
        writer.seek(4).unwrap();
        writer.write(&99u8).unwrap();

        let data = writer.into_inner().unwrap();
        assert_eq!(data, vec![42, 0, 0, 0, 0, 99]);
    }

    #[test]
    fn test_writer_seek_zero() {
        let mut writer = Writer::from_vec(Vec::new());

        writer.write(&42u8).unwrap();
        writer.seek(0).unwrap();
        writer.write(&99u8).unwrap();

        let data = writer.into_inner().unwrap();
        assert_eq!(data, vec![42, 99]);
    }

    #[test]
    fn test_writer_seek_large() {
        let mut writer = Writer::from_vec(Vec::new());

        writer.write(&1u8).unwrap();
        writer.seek(1500).unwrap();
        writer.write(&2u8).unwrap();

        let data = writer.into_inner().unwrap();
        assert_eq!(data.len(), 1502);
        assert_eq!(data[0], 1);
        assert_eq!(data[1501], 2);
        // Check that middle is zeros
        assert!(data[1..1501].iter().all(|&b| b == 0));
    }

    #[test]
    fn test_writer_clear() {
        let mut writer = Writer::from_vec(Vec::new());

        assert!(writer.is_open());
        writer.clear();
        assert!(!writer.is_open());
    }

    #[test]
    fn test_writer_not_open() {
        let mut writer = Writer::new();
        let value: u32 = 42;

        let result = writer.write(&value);
        assert!(result.is_err());
    }

    #[test]
    fn test_writer_from_writer() {
        let vec = Vec::new();
        let cursor = io::Cursor::new(vec);
        let mut writer = Writer::from_writer(cursor);

        assert!(writer.is_open());

        let value: u32 = 0x04030201;
        writer.write(&value).unwrap();

        // Note: from_writer doesn't support into_inner, so we just verify it wrote
        writer.clear();
        assert!(!writer.is_open());
    }

    #[test]
    fn test_writer_write_multiple_types() {
        let mut writer = Writer::from_vec(Vec::new());

        writer.write(&42u32).unwrap();
        writer.write(&100u64).unwrap();

        let data = writer.into_inner().unwrap();

        let mut expected = Vec::new();
        expected.extend_from_slice(&42u32.to_le_bytes());
        expected.extend_from_slice(&100u64.to_le_bytes());

        assert_eq!(data, expected);
    }

    #[test]
    fn test_writer_multiple_writes() {
        let mut writer = Writer::from_vec(Vec::new());

        writer.write(&1u8).unwrap();
        writer.write(&2u8).unwrap();
        writer.write(&3u8).unwrap();

        let data = writer.into_inner().unwrap();
        assert_eq!(data, vec![1, 2, 3]);
    }

    #[test]
    fn test_writer_default() {
        let writer = Writer::default();
        assert!(!writer.is_open());
    }

    #[test]
    fn test_writer_write_slice_multiple() {
        let mut writer = Writer::from_vec(Vec::new());

        let values1 = [1u8, 2, 3, 4];
        let values2 = [5u8, 6, 7, 8];

        writer.write_slice(&values1).unwrap();
        writer.write_slice(&values2).unwrap();

        let data = writer.into_inner().unwrap();
        assert_eq!(data, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    }
}
