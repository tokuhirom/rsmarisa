//! I/O operations for reading, writing, and memory mapping.
//!
//! Ported from: lib/marisa/grimoire/io/
//!
//! This module provides:
//! - Reader: for reading trie data from files or memory
//! - Writer: for writing trie data to files or memory
//! - Mapper: for memory-mapped file access

pub mod mapper;
pub mod reader;
pub mod writer;

pub use mapper::Mapper;
pub use reader::Reader;
pub use writer::Writer;
