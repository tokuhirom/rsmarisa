//! Base types, constants, and error definitions.
//!
//! Ported from: include/marisa/base.h
//!
//! This module defines fundamental types and constants used throughout the library,
//! including error codes, configuration flags, and invalid ID constants.

use std::fmt;

/// Word size in bits (32 or 64) based on pointer size.
#[cfg(target_pointer_width = "64")]
pub const WORD_SIZE: usize = 64;

#[cfg(target_pointer_width = "32")]
pub const WORD_SIZE: usize = 32;

/// Invalid link ID constant.
pub const INVALID_LINK_ID: u32 = u32::MAX;

/// Invalid key ID constant.
pub const INVALID_KEY_ID: u32 = u32::MAX;

/// Tail mode for suffix storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TailMode {
    /// Text mode: merges suffixes as NULL-terminated strings.
    ///
    /// Available only if suffixes don't contain NULL characters.
    /// Automatically switches to Binary mode if NULL is detected.
    TextTail = 0x01000,

    /// Binary mode: merges suffixes as byte sequences.
    ///
    /// Uses a bit vector to detect end of sequences instead of NULL.
    /// Requires more space if average suffix length > 8 bytes.
    BinaryTail = 0x02000,
}

impl Default for TailMode {
    fn default() -> Self {
        TailMode::TextTail
    }
}

/// Invalid extra value constant (UINT32_MAX >> 8).
pub const INVALID_EXTRA: u32 = u32::MAX >> 8;

/// Error codes used throughout the library.
///
/// Ported from: marisa_error_code enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// Operation succeeded (not used in practice for errors).
    Ok = 0,

    /// Object was not ready for the requested operation.
    StateError = 1,

    /// Invalid null pointer was given.
    NullError = 2,

    /// Operation tried to access an out of range address.
    BoundError = 3,

    /// Out of range value appeared in operation.
    RangeError = 4,

    /// Undefined code appeared in operation.
    CodeError = 5,

    /// Smart pointer tried to reset itself.
    ResetError = 6,

    /// Size exceeded library limitation.
    SizeError = 7,

    /// Memory allocation failed.
    MemoryError = 8,

    /// I/O operation failed.
    IoError = 9,

    /// Input was in invalid format.
    FormatError = 10,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::Ok => write!(f, "OK"),
            ErrorCode::StateError => write!(f, "STATE_ERROR"),
            ErrorCode::NullError => write!(f, "NULL_ERROR"),
            ErrorCode::BoundError => write!(f, "BOUND_ERROR"),
            ErrorCode::RangeError => write!(f, "RANGE_ERROR"),
            ErrorCode::CodeError => write!(f, "CODE_ERROR"),
            ErrorCode::ResetError => write!(f, "RESET_ERROR"),
            ErrorCode::SizeError => write!(f, "SIZE_ERROR"),
            ErrorCode::MemoryError => write!(f, "MEMORY_ERROR"),
            ErrorCode::IoError => write!(f, "IO_ERROR"),
            ErrorCode::FormatError => write!(f, "FORMAT_ERROR"),
        }
    }
}

impl std::error::Error for ErrorCode {}

/// Flags for memory mapping.
///
/// Ported from: marisa_map_flags enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapFlags(pub u32);

impl MapFlags {
    /// Specifies MAP_POPULATE.
    pub const POPULATE: MapFlags = MapFlags(1 << 0);
}

/// Number of tries in a dictionary.
///
/// A dictionary consists of 3 tries by default. More tries generally make
/// a dictionary more space-efficient but less time-efficient.
///
/// Ported from: marisa_num_tries enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumTries(pub u32);

impl NumTries {
    pub const MIN: u32 = 0x00001;
    pub const MAX: u32 = 0x0007F;
    pub const DEFAULT: u32 = 0x00003;
}

/// Cache size options for search acceleration.
///
/// Larger cache enables faster search but takes more space.
///
/// Ported from: marisa_cache_level enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CacheLevel {
    /// Huge cache size.
    Huge = 0x00080,
    /// Large cache size.
    Large = 0x00100,
    /// Normal cache size (default).
    Normal = 0x00200,
    /// Small cache size.
    Small = 0x00400,
    /// Tiny cache size.
    Tiny = 0x00800,
}

impl Default for CacheLevel {
    fn default() -> Self {
        CacheLevel::Normal
    }
}

/// Node arrangement order.
///
/// The arrangement affects matching time cost and predictive search order.
///
/// Ported from: marisa_node_order enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NodeOrder {
    /// Arranges nodes in ascending label order.
    ///
    /// Useful for predicting keys in label order.
    Label = 0x10000,

    /// Arranges nodes in descending weight order (default).
    ///
    /// Generally better choice as it enables faster matching.
    Weight = 0x20000,
}

impl Default for NodeOrder {
    fn default() -> Self {
        NodeOrder::Weight
    }
}

/// Configuration masks for extracting specific config bits.
///
/// Ported from: marisa_config_mask enum
pub mod config_mask {
    pub const NUM_TRIES: u32 = 0x0007F;
    pub const CACHE_LEVEL: u32 = 0x00F80;
    pub const TAIL_MODE: u32 = 0x0F000;
    pub const NODE_ORDER: u32 = 0xF0000;
    pub const CONFIG: u32 = 0xFFFFF;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_size() {
        // Word size should be either 32 or 64
        assert!(WORD_SIZE == 32 || WORD_SIZE == 64);
        assert_eq!(WORD_SIZE, std::mem::size_of::<usize>() * 8);
    }

    #[test]
    fn test_invalid_constants() {
        assert_eq!(INVALID_LINK_ID, u32::MAX);
        assert_eq!(INVALID_KEY_ID, u32::MAX);
        assert_eq!(INVALID_EXTRA, u32::MAX >> 8);
    }

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::Ok.to_string(), "OK");
        assert_eq!(ErrorCode::StateError.to_string(), "STATE_ERROR");
        assert_eq!(ErrorCode::IoError.to_string(), "IO_ERROR");
    }

    #[test]
    fn test_default_values() {
        assert_eq!(CacheLevel::default(), CacheLevel::Normal);
        assert_eq!(TailMode::default(), TailMode::TextTail);
        assert_eq!(NodeOrder::default(), NodeOrder::Weight);
    }
}
