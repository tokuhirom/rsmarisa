//! Reader for deserializing trie data.
//!
//! Ported from:
//! - lib/marisa/grimoire/io/reader.h
//! - lib/marisa/grimoire/io/reader.cc

use crate::base::ErrorCode;
use std::io::Read as IoRead;

/// Reader for reading trie data from files or memory.
///
/// This is a stub implementation. Full implementation will be added later.
pub struct Reader {
    _marker: std::marker::PhantomData<()>,
}

impl Reader {
    /// Creates a new reader.
    pub fn new() -> Self {
        Reader {
            _marker: std::marker::PhantomData,
        }
    }

    /// Reads a single value of type T.
    pub fn read<T>(&mut self, _value: &mut T) -> Result<(), ErrorCode> {
        // TODO: implement
        Err(ErrorCode::IoError)
    }

    /// Reads multiple values into a slice.
    pub fn read_slice<T>(&mut self, _values: &mut [T]) -> Result<(), ErrorCode> {
        // TODO: implement
        Err(ErrorCode::IoError)
    }

    /// Seeks forward by the specified number of bytes.
    pub fn seek(&mut self, _size: usize) -> Result<(), ErrorCode> {
        // TODO: implement
        Err(ErrorCode::IoError)
    }
}

impl Default for Reader {
    fn default() -> Self {
        Self::new()
    }
}
