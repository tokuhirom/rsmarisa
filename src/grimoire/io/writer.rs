//! Writer for serializing trie data.
//!
//! Ported from:
//! - lib/marisa/grimoire/io/writer.h
//! - lib/marisa/grimoire/io/writer.cc

use crate::base::ErrorCode;
use std::io::Write as IoWrite;

/// Writer for writing trie data to files or memory.
///
/// This is a stub implementation. Full implementation will be added later.
pub struct Writer {
    _marker: std::marker::PhantomData<()>,
}

impl Writer {
    /// Creates a new writer.
    pub fn new() -> Self {
        Writer {
            _marker: std::marker::PhantomData,
        }
    }

    /// Writes a single value of type T.
    pub fn write<T>(&mut self, _value: &T) -> Result<(), ErrorCode> {
        // TODO: implement
        Err(ErrorCode::IoError)
    }

    /// Writes multiple values from a slice.
    pub fn write_slice<T>(&mut self, _values: &[T]) -> Result<(), ErrorCode> {
        // TODO: implement
        Err(ErrorCode::IoError)
    }

    /// Seeks forward by the specified number of bytes.
    pub fn seek(&mut self, _size: usize) -> Result<(), ErrorCode> {
        // TODO: implement
        Err(ErrorCode::IoError)
    }
}

impl Default for Writer {
    fn default() -> Self {
        Self::new()
    }
}
