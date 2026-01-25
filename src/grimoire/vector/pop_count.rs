//! Population count utilities.
//!
//! Ported from: lib/marisa/grimoire/vector/pop-count.h

/// Counts the number of set bits in a value.
#[inline]
pub fn pop_count(x: u64) -> u32 {
    x.count_ones()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pop_count() {
        assert_eq!(pop_count(0), 0);
        assert_eq!(pop_count(1), 1);
        assert_eq!(pop_count(0b1010), 2);
        assert_eq!(pop_count(u64::MAX), 64);
    }
}
