//! Trie data structure implementation.
//!
//! Ported from: lib/marisa/grimoire/trie/
//!
//! This module provides the core trie implementation based on LOUDS
//! (Level-Order Unary Degree Sequence) with associated data structures:
//! - LOUDS trie: main trie structure
//! - Tail: suffix storage
//! - Cache: search acceleration
//! - Config: build configuration
//! - Entry, Key, Range: helper structures

pub mod cache;
pub mod config;
pub mod entry;
pub mod header;
pub mod history;
pub mod key;
pub mod louds_trie;
pub mod range;
pub mod state;
pub mod tail;
