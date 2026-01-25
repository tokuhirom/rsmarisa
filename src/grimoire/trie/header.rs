//! Trie header for file format identification.
//!
//! Ported from: lib/marisa/grimoire/trie/header.h
//!
//! The header is a simple magic string "We love Marisa." used to identify
//! valid trie files and verify file format integrity.

use crate::grimoire::io::{Mapper, Reader, Writer};

/// Size of the header in bytes.
pub const HEADER_SIZE: usize = 16;

/// Header for trie file format identification.
///
/// The header contains a magic string to verify that a file or memory region
/// contains a valid MARISA trie.
#[derive(Default)]
pub struct Header;

impl Header {
    /// Creates a new header.
    #[inline]
    pub fn new() -> Self {
        Header
    }

    /// Returns the magic header string.
    ///
    /// # Returns
    ///
    /// A 16-byte slice containing "We love Marisa.\0"
    #[inline]
    fn get_header() -> &'static [u8; HEADER_SIZE] {
        b"We love Marisa.\0"
    }

    /// Tests if the given bytes match the expected header.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes to test
    ///
    /// # Returns
    ///
    /// true if bytes match the header, false otherwise
    fn test_header(bytes: &[u8]) -> bool {
        if bytes.len() != HEADER_SIZE {
            return false;
        }
        bytes == Self::get_header()
    }

    /// Maps the header from a mapper (for memory-mapped I/O).
    ///
    /// # Arguments
    ///
    /// * `mapper` - The mapper to read from
    ///
    /// # Panics
    ///
    /// Panics if the header is invalid (TODO: should return Result)
    pub fn map(&mut self, _mapper: &mut Mapper) {
        // TODO: implement when Mapper is complete
        panic!("Header::map not yet implemented - Mapper interface incomplete");
    }

    /// Reads the header from a reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - The reader to read from
    ///
    /// # Errors
    ///
    /// Returns an error if the header is invalid or reading fails
    pub fn read(&mut self, reader: &mut Reader) -> std::io::Result<()> {
        let mut buf = [0u8; HEADER_SIZE];
        reader.read_slice(&mut buf)?;

        if !Self::test_header(&buf) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid MARISA header",
            ));
        }

        Ok(())
    }

    /// Writes the header to a writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - The writer to write to
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails
    pub fn write(&self, writer: &mut Writer) -> std::io::Result<()> {
        writer.write_slice(Self::get_header())
    }

    /// Returns the I/O size of the header.
    ///
    /// # Returns
    ///
    /// The size in bytes (always HEADER_SIZE)
    #[inline]
    pub fn io_size(&self) -> usize {
        HEADER_SIZE
    }

    /// Validates a byte slice as a valid header.
    ///
    /// This is a public API for testing headers without I/O operations.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes to validate
    ///
    /// # Returns
    ///
    /// true if the bytes represent a valid MARISA header
    pub fn validate(bytes: &[u8]) -> bool {
        Self::test_header(bytes)
    }

    /// Returns a copy of the header bytes.
    ///
    /// # Returns
    ///
    /// A 16-byte array containing the header
    pub fn bytes() -> [u8; HEADER_SIZE] {
        *Self::get_header()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_new() {
        let header = Header::new();
        assert_eq!(header.io_size(), HEADER_SIZE);
    }

    #[test]
    fn test_header_size() {
        assert_eq!(HEADER_SIZE, 16);
    }

    #[test]
    fn test_header_bytes() {
        let bytes = Header::bytes();
        assert_eq!(bytes.len(), 16);
        assert_eq!(&bytes[..15], b"We love Marisa.");
        assert_eq!(bytes[15], 0); // null terminator
    }

    #[test]
    fn test_header_validate_valid() {
        let valid_header = b"We love Marisa.\0";
        assert!(Header::validate(valid_header));
    }

    #[test]
    fn test_header_validate_invalid_content() {
        let invalid_header = b"Invalid header!\0";
        assert!(!Header::validate(invalid_header));
    }

    #[test]
    fn test_header_validate_invalid_length() {
        let short_header = b"Too short";
        assert!(!Header::validate(short_header));

        let long_header = b"This header is too long for validation";
        assert!(!Header::validate(long_header));
    }

    #[test]
    fn test_header_validate_partial_match() {
        let partial = b"We love Marisa.X"; // Last byte is wrong
        assert!(!Header::validate(partial));
    }

    #[test]
    fn test_header_default() {
        let header = Header::default();
        assert_eq!(header.io_size(), HEADER_SIZE);
    }
}
