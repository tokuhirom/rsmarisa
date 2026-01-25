//! Sorting algorithms for trie construction.
//!
//! Ported from: lib/marisa/grimoire/algorithm/sort.h

/// Sorts elements for trie construction.
pub fn sort<T: Ord>(data: &mut [T]) {
    data.sort();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort() {
        let mut data = vec![3, 1, 4, 1, 5, 9, 2, 6];
        sort(&mut data);
        assert_eq!(data, vec![1, 1, 2, 3, 4, 5, 6, 9]);
    }
}
