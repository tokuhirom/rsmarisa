//! Memory-mapped file access.
//!
//! Ported from:
//! - lib/marisa/grimoire/io/mapper.h
//! - lib/marisa/grimoire/io/mapper.cc

use crate::base::ErrorCode;

/// Mapper for memory-mapped file access.
///
/// This is a stub implementation. Full implementation will be added later.
pub struct Mapper {
    _marker: std::marker::PhantomData<()>,
}

impl Mapper {
    /// Creates a new mapper.
    pub fn new() -> Self {
        Mapper {
            _marker: std::marker::PhantomData,
        }
    }

    /// Maps a single value of type T.
    pub fn map<T>(&mut self, _value: &mut T) -> Result<(), ErrorCode> {
        // TODO: implement
        Err(ErrorCode::IoError)
    }

    /// Maps a slice of values.
    pub fn map_slice<'a, T>(&mut self, _slice: &mut &'a [T]) -> Result<(), ErrorCode> {
        // TODO: implement
        Err(ErrorCode::IoError)
    }

    /// Seeks forward by the specified number of bytes.
    pub fn seek(&mut self, _size: usize) -> Result<(), ErrorCode> {
        // TODO: implement
        Err(ErrorCode::IoError)
    }
}

impl Default for Mapper {
    fn default() -> Self {
        Self::new()
    }
}
