//! Cache structure for trie construction.
//!
//! Ported from: lib/marisa/grimoire/trie/cache.h
//!
//! Cache stores temporary node information during trie construction,
//! including parent/child relationships and either link data (base + extra)
//! or weight values.

/// Link or weight data stored in cache.
#[derive(Debug, Clone, Copy, PartialEq)]
enum LinkOrWeight {
    /// Link value containing base (low 8 bits) and extra (high 24 bits).
    Link(u32),
    /// Weight value for sorting.
    Weight(f32),
}

impl Default for LinkOrWeight {
    fn default() -> Self {
        LinkOrWeight::Weight(f32::MIN)
    }
}

/// Cache for storing temporary node information during trie construction.
///
/// Cache holds parent and child node indices, along with either link
/// information (base byte + extra data) or weight for sorting.
#[derive(Debug, Clone, Copy, Default)]
pub struct Cache {
    /// Parent node index.
    parent: u32,
    /// Child node index.
    child: u32,
    /// Link data or weight.
    union: LinkOrWeight,
}

impl Cache {
    /// Creates a new cache with default values.
    pub fn new() -> Self {
        Cache {
            parent: 0,
            child: 0,
            union: LinkOrWeight::default(),
        }
    }

    /// Sets the parent node index.
    ///
    /// # Arguments
    ///
    /// * `parent` - Parent node index
    ///
    /// # Panics
    ///
    /// Panics if parent > u32::MAX
    #[inline]
    pub fn set_parent(&mut self, parent: usize) {
        assert!(parent <= u32::MAX as usize, "Parent exceeds u32::MAX");
        self.parent = parent as u32;
    }

    /// Sets the child node index.
    ///
    /// # Arguments
    ///
    /// * `child` - Child node index
    ///
    /// # Panics
    ///
    /// Panics if child > u32::MAX
    #[inline]
    pub fn set_child(&mut self, child: usize) {
        assert!(child <= u32::MAX as usize, "Child exceeds u32::MAX");
        self.child = child as u32;
    }

    /// Sets the base byte (low 8 bits of link).
    ///
    /// # Arguments
    ///
    /// * `base` - Base byte value
    #[inline]
    pub fn set_base(&mut self, base: u8) {
        let current_link = match self.union {
            LinkOrWeight::Link(link) => link,
            _ => 0,
        };
        self.union = LinkOrWeight::Link((current_link & !0xFF) | (base as u32));
    }

    /// Sets the extra data (high 24 bits of link).
    ///
    /// # Arguments
    ///
    /// * `extra` - Extra data value
    ///
    /// # Panics
    ///
    /// Panics if extra doesn't fit in 24 bits
    #[inline]
    pub fn set_extra(&mut self, extra: usize) {
        assert!(extra <= (u32::MAX >> 8) as usize, "Extra too large");
        let current_link = match self.union {
            LinkOrWeight::Link(link) => link,
            _ => 0,
        };
        self.union = LinkOrWeight::Link((current_link & 0xFF) | ((extra as u32) << 8));
    }

    /// Sets the weight.
    ///
    /// # Arguments
    ///
    /// * `weight` - Weight value
    #[inline]
    pub fn set_weight(&mut self, weight: f32) {
        self.union = LinkOrWeight::Weight(weight);
    }

    /// Returns the parent node index.
    #[inline]
    pub fn parent(&self) -> usize {
        self.parent as usize
    }

    /// Returns the child node index.
    #[inline]
    pub fn child(&self) -> usize {
        self.child as usize
    }

    /// Returns the base byte.
    ///
    /// # Panics
    ///
    /// Panics if weight was set instead of link
    #[inline]
    pub fn base(&self) -> u8 {
        match self.union {
            LinkOrWeight::Link(link) => (link & 0xFF) as u8,
            _ => panic!("Link not set"),
        }
    }

    /// Returns the extra data.
    ///
    /// # Panics
    ///
    /// Panics if weight was set instead of link
    #[inline]
    pub fn extra(&self) -> usize {
        match self.union {
            LinkOrWeight::Link(link) => (link >> 8) as usize,
            _ => panic!("Link not set"),
        }
    }

    /// Returns the label (same as base, cast to char).
    ///
    /// # Panics
    ///
    /// Panics if weight was set instead of link
    #[inline]
    pub fn label(&self) -> u8 {
        self.base()
    }

    /// Returns the full link value.
    ///
    /// # Panics
    ///
    /// Panics if weight was set instead of link
    #[inline]
    pub fn link(&self) -> usize {
        match self.union {
            LinkOrWeight::Link(link) => link as usize,
            _ => panic!("Link not set"),
        }
    }

    /// Returns the weight.
    ///
    /// # Panics
    ///
    /// Panics if link was set instead of weight
    #[inline]
    pub fn weight(&self) -> f32 {
        match self.union {
            LinkOrWeight::Weight(w) => w,
            _ => panic!("Weight not set"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_new() {
        let cache = Cache::new();
        assert_eq!(cache.parent(), 0);
        assert_eq!(cache.child(), 0);
    }

    #[test]
    fn test_cache_parent_child() {
        let mut cache = Cache::new();
        cache.set_parent(100);
        cache.set_child(200);

        assert_eq!(cache.parent(), 100);
        assert_eq!(cache.child(), 200);
    }

    #[test]
    fn test_cache_base_extra() {
        let mut cache = Cache::new();
        cache.set_base(0x42);
        cache.set_extra(0x123456);

        assert_eq!(cache.base(), 0x42);
        assert_eq!(cache.extra(), 0x123456);
        assert_eq!(cache.label(), 0x42);
        assert_eq!(cache.link(), 0x12345642);
    }

    #[test]
    fn test_cache_base_preserves_extra() {
        let mut cache = Cache::new();
        cache.set_extra(0x123456);
        cache.set_base(0x42);

        assert_eq!(cache.base(), 0x42);
        assert_eq!(cache.extra(), 0x123456);
    }

    #[test]
    fn test_cache_extra_preserves_base() {
        let mut cache = Cache::new();
        cache.set_base(0x42);
        cache.set_extra(0x123456);

        assert_eq!(cache.base(), 0x42);
        assert_eq!(cache.extra(), 0x123456);
    }

    #[test]
    fn test_cache_weight() {
        let mut cache = Cache::new();
        cache.set_weight(3.14);

        assert_eq!(cache.weight(), 3.14);
    }

    #[test]
    fn test_cache_weight_replaces_link() {
        let mut cache = Cache::new();
        cache.set_base(0x42);
        cache.set_extra(0x123456);
        cache.set_weight(2.5);

        assert_eq!(cache.weight(), 2.5);
    }

    #[test]
    fn test_cache_link_replaces_weight() {
        let mut cache = Cache::new();
        cache.set_weight(2.5);
        cache.set_base(0x42);
        cache.set_extra(0x123456);

        assert_eq!(cache.base(), 0x42);
        assert_eq!(cache.extra(), 0x123456);
    }

    #[test]
    fn test_cache_max_values() {
        let mut cache = Cache::new();
        cache.set_parent(u32::MAX as usize);
        cache.set_child(u32::MAX as usize);

        assert_eq!(cache.parent(), u32::MAX as usize);
        assert_eq!(cache.child(), u32::MAX as usize);
    }

    #[test]
    fn test_cache_base_all_bits() {
        let mut cache = Cache::new();
        cache.set_base(0xFF);

        assert_eq!(cache.base(), 0xFF);
        assert_eq!(cache.extra(), 0);
    }

    #[test]
    fn test_cache_extra_max() {
        let mut cache = Cache::new();
        cache.set_extra((u32::MAX >> 8) as usize);

        assert_eq!(cache.extra(), (u32::MAX >> 8) as usize);
    }

    #[test]
    fn test_cache_default() {
        let cache = Cache::default();
        assert_eq!(cache.parent(), 0);
        assert_eq!(cache.child(), 0);
    }

    #[test]
    fn test_cache_clone() {
        let mut cache1 = Cache::new();
        cache1.set_parent(10);
        cache1.set_child(20);
        cache1.set_base(0x42);

        let cache2 = cache1;
        assert_eq!(cache2.parent(), 10);
        assert_eq!(cache2.child(), 20);
        assert_eq!(cache2.base(), 0x42);
    }
}
