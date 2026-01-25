//! Rank index for accelerating rank operations.
//!
//! Ported from:
//! - lib/marisa/grimoire/vector/rank-index.h
//!
//! RankIndex stores rank information for a 512-bit block using bit packing.
//! It contains one absolute rank value and 7 relative rank values.

/// Rank index for bit vector rank operation acceleration.
///
/// This structure stores rank information for efficient rank queries on bit vectors.
/// It uses bit packing to store one absolute rank (32 bits) and 7 relative ranks
/// (packed into two 32-bit values).
#[derive(Debug, Clone, Copy, Default)]
pub struct RankIndex {
    /// Absolute rank count (full 32 bits).
    abs: u32,
    /// Lower relative ranks (rel1, rel2, rel3, rel4) bit-packed.
    rel_lo: u32,
    /// Higher relative ranks (rel5, rel6, rel7) bit-packed.
    rel_hi: u32,
}

impl RankIndex {
    /// Creates a new rank index with all values set to zero.
    #[inline]
    pub fn new() -> Self {
        RankIndex {
            abs: 0,
            rel_lo: 0,
            rel_hi: 0,
        }
    }

    /// Sets the absolute rank value.
    #[inline]
    pub fn set_abs(&mut self, value: usize) {
        debug_assert!(value <= u32::MAX as usize);
        self.abs = value as u32;
    }

    /// Sets relative rank 1 (bits 0-6 of rel_lo).
    #[inline]
    pub fn set_rel1(&mut self, value: usize) {
        debug_assert!(value <= 64);
        self.rel_lo = (self.rel_lo & !0x7F) | ((value as u32) & 0x7F);
    }

    /// Sets relative rank 2 (bits 7-14 of rel_lo).
    #[inline]
    pub fn set_rel2(&mut self, value: usize) {
        debug_assert!(value <= 128);
        self.rel_lo = (self.rel_lo & !(0xFF << 7)) | (((value as u32) & 0xFF) << 7);
    }

    /// Sets relative rank 3 (bits 15-22 of rel_lo).
    #[inline]
    pub fn set_rel3(&mut self, value: usize) {
        debug_assert!(value <= 192);
        self.rel_lo = (self.rel_lo & !(0xFF << 15)) | (((value as u32) & 0xFF) << 15);
    }

    /// Sets relative rank 4 (bits 23-31 of rel_lo).
    #[inline]
    pub fn set_rel4(&mut self, value: usize) {
        debug_assert!(value <= 256);
        self.rel_lo = (self.rel_lo & !(0x1FF << 23)) | (((value as u32) & 0x1FF) << 23);
    }

    /// Sets relative rank 5 (bits 0-8 of rel_hi).
    #[inline]
    pub fn set_rel5(&mut self, value: usize) {
        debug_assert!(value <= 320);
        self.rel_hi = (self.rel_hi & !0x1FF) | ((value as u32) & 0x1FF);
    }

    /// Sets relative rank 6 (bits 9-17 of rel_hi).
    #[inline]
    pub fn set_rel6(&mut self, value: usize) {
        debug_assert!(value <= 384);
        self.rel_hi = (self.rel_hi & !(0x1FF << 9)) | (((value as u32) & 0x1FF) << 9);
    }

    /// Sets relative rank 7 (bits 18-26 of rel_hi).
    #[inline]
    pub fn set_rel7(&mut self, value: usize) {
        debug_assert!(value <= 448);
        self.rel_hi = (self.rel_hi & !(0x1FF << 18)) | (((value as u32) & 0x1FF) << 18);
    }

    /// Returns the absolute rank value.
    #[inline]
    pub fn abs(&self) -> usize {
        self.abs as usize
    }

    /// Returns relative rank 1.
    #[inline]
    pub fn rel1(&self) -> usize {
        (self.rel_lo & 0x7F) as usize
    }

    /// Returns relative rank 2.
    #[inline]
    pub fn rel2(&self) -> usize {
        ((self.rel_lo >> 7) & 0xFF) as usize
    }

    /// Returns relative rank 3.
    #[inline]
    pub fn rel3(&self) -> usize {
        ((self.rel_lo >> 15) & 0xFF) as usize
    }

    /// Returns relative rank 4.
    #[inline]
    pub fn rel4(&self) -> usize {
        ((self.rel_lo >> 23) & 0x1FF) as usize
    }

    /// Returns relative rank 5.
    #[inline]
    pub fn rel5(&self) -> usize {
        (self.rel_hi & 0x1FF) as usize
    }

    /// Returns relative rank 6.
    #[inline]
    pub fn rel6(&self) -> usize {
        ((self.rel_hi >> 9) & 0x1FF) as usize
    }

    /// Returns relative rank 7.
    #[inline]
    pub fn rel7(&self) -> usize {
        ((self.rel_hi >> 18) & 0x1FF) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_index_default() {
        let rank = RankIndex::new();
        assert_eq!(rank.abs(), 0);
        assert_eq!(rank.rel1(), 0);
        assert_eq!(rank.rel2(), 0);
        assert_eq!(rank.rel3(), 0);
        assert_eq!(rank.rel4(), 0);
        assert_eq!(rank.rel5(), 0);
        assert_eq!(rank.rel6(), 0);
        assert_eq!(rank.rel7(), 0);
    }

    #[test]
    fn test_rank_index_abs() {
        let mut rank = RankIndex::new();
        rank.set_abs(12345);
        assert_eq!(rank.abs(), 12345);
    }

    #[test]
    fn test_rank_index_rel1() {
        let mut rank = RankIndex::new();
        rank.set_rel1(64);
        assert_eq!(rank.rel1(), 64);
    }

    #[test]
    fn test_rank_index_rel2() {
        let mut rank = RankIndex::new();
        rank.set_rel2(128);
        assert_eq!(rank.rel2(), 128);
    }

    #[test]
    fn test_rank_index_multiple() {
        let mut rank = RankIndex::new();
        rank.set_abs(1000);
        rank.set_rel1(10);
        rank.set_rel2(20);
        rank.set_rel3(30);
        rank.set_rel4(40);
        rank.set_rel5(50);
        rank.set_rel6(60);
        rank.set_rel7(70);

        assert_eq!(rank.abs(), 1000);
        assert_eq!(rank.rel1(), 10);
        assert_eq!(rank.rel2(), 20);
        assert_eq!(rank.rel3(), 30);
        assert_eq!(rank.rel4(), 40);
        assert_eq!(rank.rel5(), 50);
        assert_eq!(rank.rel6(), 60);
        assert_eq!(rank.rel7(), 70);
    }
}
