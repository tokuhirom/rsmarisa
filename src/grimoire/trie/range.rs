//! Range structures for trie construction.
//!
//! Ported from: lib/marisa/grimoire/trie/range.h
//!
//! This module provides Range and WeightedRange structures used during
//! trie construction to manage ranges of keys being processed.

/// Range representing a segment of keys during trie construction.
///
/// A Range tracks a contiguous segment of keys with a begin index,
/// end index, and the current position within the key string.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Range {
    /// Starting index of the range (inclusive).
    begin: u32,
    /// Ending index of the range (exclusive).
    end: u32,
    /// Current position in the key string.
    key_pos: u32,
}

impl Range {
    /// Creates a new range with default values (all zeros).
    #[inline]
    pub fn new() -> Self {
        Range {
            begin: 0,
            end: 0,
            key_pos: 0,
        }
    }

    /// Sets the beginning index.
    ///
    /// # Arguments
    ///
    /// * `begin` - The starting index (must fit in u32)
    ///
    /// # Panics
    ///
    /// Panics if begin > u32::MAX
    #[inline]
    pub fn set_begin(&mut self, begin: usize) {
        assert!(begin <= u32::MAX as usize, "begin exceeds u32::MAX");
        self.begin = begin as u32;
    }

    /// Sets the ending index.
    ///
    /// # Arguments
    ///
    /// * `end` - The ending index (must fit in u32)
    ///
    /// # Panics
    ///
    /// Panics if end > u32::MAX
    #[inline]
    pub fn set_end(&mut self, end: usize) {
        assert!(end <= u32::MAX as usize, "end exceeds u32::MAX");
        self.end = end as u32;
    }

    /// Sets the key position.
    ///
    /// # Arguments
    ///
    /// * `key_pos` - The position in the key string (must fit in u32)
    ///
    /// # Panics
    ///
    /// Panics if key_pos > u32::MAX
    #[inline]
    pub fn set_key_pos(&mut self, key_pos: usize) {
        assert!(key_pos <= u32::MAX as usize, "key_pos exceeds u32::MAX");
        self.key_pos = key_pos as u32;
    }

    /// Returns the beginning index.
    #[inline]
    pub fn begin(&self) -> usize {
        self.begin as usize
    }

    /// Returns the ending index.
    #[inline]
    pub fn end(&self) -> usize {
        self.end as usize
    }

    /// Returns the key position.
    #[inline]
    pub fn key_pos(&self) -> usize {
        self.key_pos as usize
    }
}

/// Creates a new Range with the given values.
///
/// # Arguments
///
/// * `begin` - Starting index
/// * `end` - Ending index
/// * `key_pos` - Key position
///
/// # Returns
///
/// A new Range with the specified values
#[inline]
pub fn make_range(begin: usize, end: usize, key_pos: usize) -> Range {
    let mut range = Range::new();
    range.set_begin(begin);
    range.set_end(end);
    range.set_key_pos(key_pos);
    range
}

/// Weighted range for sorting during trie construction.
///
/// WeightedRange extends Range with a weight value used for ordering
/// keys by frequency or other criteria during construction.
#[derive(Debug, Clone, Copy, Default)]
pub struct WeightedRange {
    /// The underlying range.
    range: Range,
    /// Weight for sorting (typically key frequency).
    weight: f32,
}

impl WeightedRange {
    /// Creates a new weighted range with default values.
    #[inline]
    pub fn new() -> Self {
        WeightedRange {
            range: Range::new(),
            weight: 0.0,
        }
    }

    /// Sets the underlying range.
    #[inline]
    pub fn set_range(&mut self, range: Range) {
        self.range = range;
    }

    /// Sets the beginning index.
    #[inline]
    pub fn set_begin(&mut self, begin: usize) {
        self.range.set_begin(begin);
    }

    /// Sets the ending index.
    #[inline]
    pub fn set_end(&mut self, end: usize) {
        self.range.set_end(end);
    }

    /// Sets the key position.
    #[inline]
    pub fn set_key_pos(&mut self, key_pos: usize) {
        self.range.set_key_pos(key_pos);
    }

    /// Sets the weight.
    #[inline]
    pub fn set_weight(&mut self, weight: f32) {
        self.weight = weight;
    }

    /// Returns a reference to the underlying range.
    #[inline]
    pub fn range(&self) -> &Range {
        &self.range
    }

    /// Returns the beginning index.
    #[inline]
    pub fn begin(&self) -> usize {
        self.range.begin()
    }

    /// Returns the ending index.
    #[inline]
    pub fn end(&self) -> usize {
        self.range.end()
    }

    /// Returns the key position.
    #[inline]
    pub fn key_pos(&self) -> usize {
        self.range.key_pos()
    }

    /// Returns the weight.
    #[inline]
    pub fn weight(&self) -> f32 {
        self.weight
    }
}

impl PartialEq for WeightedRange {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

impl Eq for WeightedRange {}

impl PartialOrd for WeightedRange {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.weight.partial_cmp(&other.weight)
    }
}

impl Ord for WeightedRange {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // For NaN handling, treat NaN as less than everything
        self.weight
            .partial_cmp(&other.weight)
            .unwrap_or(std::cmp::Ordering::Less)
    }
}

/// Creates a new WeightedRange with the given values.
///
/// # Arguments
///
/// * `begin` - Starting index
/// * `end` - Ending index
/// * `key_pos` - Key position
/// * `weight` - Weight value
///
/// # Returns
///
/// A new WeightedRange with the specified values
#[inline]
pub fn make_weighted_range(begin: usize, end: usize, key_pos: usize, weight: f32) -> WeightedRange {
    let mut range = WeightedRange::new();
    range.set_begin(begin);
    range.set_end(end);
    range.set_key_pos(key_pos);
    range.set_weight(weight);
    range
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_new() {
        let range = Range::new();
        assert_eq!(range.begin(), 0);
        assert_eq!(range.end(), 0);
        assert_eq!(range.key_pos(), 0);
    }

    #[test]
    fn test_range_setters() {
        let mut range = Range::new();
        range.set_begin(10);
        range.set_end(20);
        range.set_key_pos(5);

        assert_eq!(range.begin(), 10);
        assert_eq!(range.end(), 20);
        assert_eq!(range.key_pos(), 5);
    }

    #[test]
    fn test_make_range() {
        let range = make_range(100, 200, 50);
        assert_eq!(range.begin(), 100);
        assert_eq!(range.end(), 200);
        assert_eq!(range.key_pos(), 50);
    }

    #[test]
    fn test_range_max_values() {
        let mut range = Range::new();
        range.set_begin(u32::MAX as usize);
        range.set_end(u32::MAX as usize);
        range.set_key_pos(u32::MAX as usize);

        assert_eq!(range.begin(), u32::MAX as usize);
        assert_eq!(range.end(), u32::MAX as usize);
        assert_eq!(range.key_pos(), u32::MAX as usize);
    }

    #[test]
    fn test_weighted_range_new() {
        let wrange = WeightedRange::new();
        assert_eq!(wrange.begin(), 0);
        assert_eq!(wrange.end(), 0);
        assert_eq!(wrange.key_pos(), 0);
        assert_eq!(wrange.weight(), 0.0);
    }

    #[test]
    fn test_weighted_range_setters() {
        let mut wrange = WeightedRange::new();
        wrange.set_begin(10);
        wrange.set_end(20);
        wrange.set_key_pos(5);
        wrange.set_weight(3.14);

        assert_eq!(wrange.begin(), 10);
        assert_eq!(wrange.end(), 20);
        assert_eq!(wrange.key_pos(), 5);
        assert_eq!(wrange.weight(), 3.14);
    }

    #[test]
    fn test_weighted_range_set_range() {
        let range = make_range(100, 200, 50);
        let mut wrange = WeightedRange::new();
        wrange.set_range(range);
        wrange.set_weight(2.5);

        assert_eq!(wrange.begin(), 100);
        assert_eq!(wrange.end(), 200);
        assert_eq!(wrange.key_pos(), 50);
        assert_eq!(wrange.weight(), 2.5);
    }

    #[test]
    fn test_make_weighted_range() {
        let wrange = make_weighted_range(100, 200, 50, 1.5);
        assert_eq!(wrange.begin(), 100);
        assert_eq!(wrange.end(), 200);
        assert_eq!(wrange.key_pos(), 50);
        assert_eq!(wrange.weight(), 1.5);
    }

    #[test]
    fn test_weighted_range_ordering() {
        let wrange1 = make_weighted_range(0, 10, 0, 1.0);
        let wrange2 = make_weighted_range(10, 20, 0, 2.0);
        let wrange3 = make_weighted_range(20, 30, 0, 1.5);

        assert!(wrange1 < wrange2);
        assert!(wrange2 > wrange1);
        assert!(wrange1 < wrange3);
        assert!(wrange3 < wrange2);
    }

    #[test]
    fn test_weighted_range_equality() {
        let wrange1 = make_weighted_range(0, 10, 0, 1.0);
        let wrange2 = make_weighted_range(10, 20, 0, 1.0);
        let wrange3 = make_weighted_range(0, 10, 0, 2.0);

        // Equality is based on weight only
        assert_eq!(wrange1, wrange2);
        assert_ne!(wrange1, wrange3);
    }

    #[test]
    fn test_weighted_range_sorting() {
        let mut ranges = vec![
            make_weighted_range(0, 10, 0, 3.0),
            make_weighted_range(10, 20, 0, 1.0),
            make_weighted_range(20, 30, 0, 2.0),
        ];

        ranges.sort();

        assert_eq!(ranges[0].weight(), 1.0);
        assert_eq!(ranges[1].weight(), 2.0);
        assert_eq!(ranges[2].weight(), 3.0);
    }

    #[test]
    fn test_range_default() {
        let range = Range::default();
        assert_eq!(range.begin(), 0);
        assert_eq!(range.end(), 0);
        assert_eq!(range.key_pos(), 0);
    }

    #[test]
    fn test_weighted_range_default() {
        let wrange = WeightedRange::default();
        assert_eq!(wrange.begin(), 0);
        assert_eq!(wrange.weight(), 0.0);
    }

    #[test]
    fn test_range_clone() {
        let range1 = make_range(10, 20, 5);
        let range2 = range1;
        assert_eq!(range1, range2);
    }

    #[test]
    fn test_weighted_range_clone() {
        let wrange1 = make_weighted_range(10, 20, 5, 1.5);
        let wrange2 = wrange1;
        assert_eq!(wrange1.weight(), wrange2.weight());
        assert_eq!(wrange1.begin(), wrange2.begin());
    }
}
