//! Generic vector wrapper.
//!
//! Ported from: lib/marisa/grimoire/vector/vector.h

/// Generic vector for internal use.
pub struct Vector<T> {
    data: Vec<T>,
}

impl<T> Vector<T> {
    /// Creates a new vector.
    pub fn new() -> Self {
        Vector { data: Vec::new() }
    }
}

impl<T> Default for Vector<T> {
    fn default() -> Self {
        Self::new()
    }
}
