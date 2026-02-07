//! Sorting algorithms for trie construction.
//!
//! Ported from: lib/marisa/grimoire/algorithm/sort.h
//!
//! This module implements depth-based string sorting using a hybrid approach
//! of quicksort and insertion sort, optimized for trie construction.

/// Threshold for switching from quicksort to insertion sort.
const INSERTION_SORT_THRESHOLD: usize = 16;

/// Trait for types that can be sorted by this algorithm.
///
/// Types must support indexed access to bytes and provide a length.
pub trait Sortable {
    /// Returns the byte at the given index, or None if index >= length.
    fn get(&self, index: usize) -> Option<u8>;

    /// Returns the length of the sortable element.
    fn length(&self) -> usize;
}

/// Gets the label (byte value) at the specified depth.
///
/// Returns -1 if depth >= length (end-of-string marker).
#[inline]
fn get_label<T: Sortable>(unit: &T, depth: usize) -> i32 {
    if depth < unit.length() {
        unit.get(depth).unwrap() as i32
    } else {
        -1
    }
}

/// Computes the median of three labels for pivot selection.
fn median<T: Sortable>(a: &T, b: &T, c: &T, depth: usize) -> i32 {
    let x = get_label(a, depth);
    let y = get_label(b, depth);
    let z = get_label(c, depth);

    if x < y {
        if y < z {
            y
        } else if x < z {
            z
        } else {
            x
        }
    } else if x < z {
        x
    } else if y < z {
        z
    } else {
        y
    }
}

/// Compares two sortable elements starting from the given depth.
///
/// Returns:
/// - Negative if lhs < rhs
/// - 0 if lhs == rhs
/// - Positive if lhs > rhs
fn compare<T: Sortable>(lhs: &T, rhs: &T, depth: usize) -> i32 {
    let mut i = depth;
    while i < lhs.length() {
        if i == rhs.length() {
            return 1;
        }
        let lhs_byte = lhs.get(i).unwrap();
        let rhs_byte = rhs.get(i).unwrap();
        if lhs_byte != rhs_byte {
            return lhs_byte as i32 - rhs_byte as i32;
        }
        i += 1;
    }

    if lhs.length() == rhs.length() {
        0
    } else if lhs.length() < rhs.length() {
        -1
    } else {
        1
    }
}

/// Insertion sort for small ranges.
///
/// Returns the count of unique string prefixes up to the given depth.
fn insertion_sort<T: Sortable>(data: &mut [T], depth: usize) -> usize {
    if data.is_empty() {
        return 0;
    }

    let mut count = 1;
    for i in 1..data.len() {
        let mut result = 0;
        for j in (1..=i).rev() {
            result = compare(&data[j - 1], &data[j], depth);
            if result <= 0 {
                break;
            }
            data.swap(j - 1, j);
        }
        if result != 0 {
            count += 1;
        }
    }
    count
}

/// Depth-based quicksort implementation.
///
/// This is a three-way quicksort optimized for string sorting, using
/// the depth parameter to compare strings character by character.
///
/// Returns the count of unique string prefixes.
fn sort_impl<T: Sortable>(data: &mut [T], depth: usize) -> usize {
    let mut count = 0;
    let mut l = 0;
    let mut r = data.len();

    while (r - l) > INSERTION_SORT_THRESHOLD {
        let mut pl = l;
        let mut pr = r;
        let mut pivot_l = l;
        let mut pivot_r = r;

        // Select pivot using median-of-three
        let pivot = median(&data[l], &data[l + (r - l) / 2], &data[r - 1], depth);

        loop {
            // Move pl forward past elements less than or equal to pivot
            while pl < pr {
                let label = get_label(&data[pl], depth);
                if label > pivot {
                    break;
                } else if label == pivot {
                    data.swap(pl, pivot_l);
                    pivot_l += 1;
                }
                pl += 1;
            }

            // Move pr backward past elements greater than or equal to pivot
            while pl < pr {
                pr -= 1;
                let label = get_label(&data[pr], depth);
                if label < pivot {
                    break;
                } else if label == pivot {
                    pivot_r -= 1;
                    data.swap(pr, pivot_r);
                }
            }

            if pl >= pr {
                break;
            }

            data.swap(pl, pr);
            pl += 1;
        }

        // Move pivot elements to the middle
        while pivot_l > l {
            pivot_l -= 1;
            pl -= 1;
            data.swap(pivot_l, pl);
        }
        while pivot_r < r {
            data.swap(pivot_r, pr);
            pivot_r += 1;
            pr += 1;
        }

        // Recursively sort partitions
        if ((pl - l) > (pr - pl)) || ((r - pr) > (pr - pl)) {
            // Middle partition (equal to pivot)
            if pr - pl == 1 {
                count += 1;
            } else if pr - pl > 1 {
                if pivot == -1 {
                    count += 1;
                } else {
                    count += sort_impl(&mut data[pl..pr], depth + 1);
                }
            }

            // Choose smaller partition to recurse on
            if (pl - l) < (r - pr) {
                if pl - l == 1 {
                    count += 1;
                } else if pl - l > 1 {
                    count += sort_impl(&mut data[l..pl], depth);
                }
                l = pr;
            } else {
                if r - pr == 1 {
                    count += 1;
                } else if r - pr > 1 {
                    count += sort_impl(&mut data[pr..r], depth);
                }
                r = pl;
            }
        } else {
            // Recurse on left partition
            if pl - l == 1 {
                count += 1;
            } else if pl - l > 1 {
                count += sort_impl(&mut data[l..pl], depth);
            }

            // Recurse on right partition
            if r - pr == 1 {
                count += 1;
            } else if r - pr > 1 {
                count += sort_impl(&mut data[pr..r], depth);
            }

            // Continue with middle partition
            l = pl;
            r = pr;
            if pr - pl == 1 {
                count += 1;
            } else if pr - pl > 1 {
                if pivot == -1 {
                    l = r;
                    count += 1;
                } else {
                    // Continue loop with increased depth
                    let mid_count = sort_impl(&mut data[l..r], depth + 1);
                    count += mid_count;
                    break;
                }
            }
        }
    }

    // Use insertion sort for small ranges
    if r - l > 1 {
        count += insertion_sort(&mut data[l..r], depth);
    }

    count
}

/// Sorts a slice of sortable elements.
///
/// This function implements a depth-based string sorting algorithm
/// optimized for trie construction. It returns the count of unique
/// string prefixes found during sorting.
///
/// # Arguments
///
/// * `data` - Mutable slice of elements to sort
///
/// # Returns
///
/// The count of unique string prefixes
pub fn sort<T: Sortable>(data: &mut [T]) -> usize {
    sort_impl(data, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple wrapper for testing
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestString {
        data: Vec<u8>,
    }

    impl TestString {
        fn new(s: &str) -> Self {
            TestString {
                data: s.as_bytes().to_vec(),
            }
        }
    }

    impl Sortable for TestString {
        fn get(&self, index: usize) -> Option<u8> {
            self.data.get(index).copied()
        }

        fn length(&self) -> usize {
            self.data.len()
        }
    }

    #[test]
    fn test_get_label() {
        let s = TestString::new("hello");
        assert_eq!(get_label(&s, 0), b'h' as i32);
        assert_eq!(get_label(&s, 4), b'o' as i32);
        assert_eq!(get_label(&s, 5), -1);
        assert_eq!(get_label(&s, 10), -1);
    }

    #[test]
    fn test_median() {
        let a = TestString::new("apple");
        let b = TestString::new("banana");
        let c = TestString::new("cherry");

        // First chars: 'a', 'b', 'c' -> median is 'b'
        assert_eq!(median(&a, &b, &c, 0), b'b' as i32);
    }

    #[test]
    fn test_compare() {
        let a = TestString::new("apple");
        let b = TestString::new("banana");
        let c = TestString::new("apple");

        assert!(compare(&a, &b, 0) < 0); // apple < banana
        assert!(compare(&b, &a, 0) > 0); // banana > apple
        assert_eq!(compare(&a, &c, 0), 0); // apple == apple
    }

    #[test]
    fn test_compare_with_depth() {
        let a = TestString::new("apple");
        let b = TestString::new("apply");
        let c = TestString::new("application");

        // Comparing from start: "apple" < "apply" because 'e' < 'y'
        assert!(compare(&a, &b, 0) < 0);

        // Comparing from index 4: 'e' < 'y'
        assert!(compare(&a, &b, 4) < 0);

        // Comparing "apple" vs "application" from index 3: "le" < "lication"
        assert!(compare(&a, &c, 3) < 0);
    }

    #[test]
    fn test_compare_prefix() {
        let a = TestString::new("app");
        let b = TestString::new("apple");

        assert!(compare(&a, &b, 0) < 0); // Shorter is less
        assert!(compare(&b, &a, 0) > 0); // Longer is greater
    }

    #[test]
    fn test_insertion_sort_simple() {
        let mut data = vec![
            TestString::new("cherry"),
            TestString::new("apple"),
            TestString::new("banana"),
        ];

        insertion_sort(&mut data, 0);

        assert_eq!(data[0].data, b"apple");
        assert_eq!(data[1].data, b"banana");
        assert_eq!(data[2].data, b"cherry");
    }

    #[test]
    fn test_insertion_sort_empty() {
        let mut data: Vec<TestString> = vec![];
        let count = insertion_sort(&mut data, 0);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_sort_simple() {
        let mut data = vec![
            TestString::new("cherry"),
            TestString::new("apple"),
            TestString::new("banana"),
            TestString::new("date"),
        ];

        sort(&mut data);

        assert_eq!(data[0].data, b"apple");
        assert_eq!(data[1].data, b"banana");
        assert_eq!(data[2].data, b"cherry");
        assert_eq!(data[3].data, b"date");
    }

    #[test]
    fn test_sort_with_common_prefixes() {
        let mut data = vec![
            TestString::new("test"),
            TestString::new("testing"),
            TestString::new("tester"),
            TestString::new("tea"),
        ];

        sort(&mut data);

        assert_eq!(data[0].data, b"tea");
        assert_eq!(data[1].data, b"test");
        assert_eq!(data[2].data, b"tester");
        assert_eq!(data[3].data, b"testing");
    }

    #[test]
    fn test_sort_duplicates() {
        let mut data = vec![
            TestString::new("apple"),
            TestString::new("banana"),
            TestString::new("apple"),
            TestString::new("banana"),
        ];

        sort(&mut data);

        assert_eq!(data[0].data, b"apple");
        assert_eq!(data[1].data, b"apple");
        assert_eq!(data[2].data, b"banana");
        assert_eq!(data[3].data, b"banana");
    }

    #[test]
    fn test_sort_already_sorted() {
        let mut data = vec![
            TestString::new("apple"),
            TestString::new("banana"),
            TestString::new("cherry"),
        ];

        sort(&mut data);

        assert_eq!(data[0].data, b"apple");
        assert_eq!(data[1].data, b"banana");
        assert_eq!(data[2].data, b"cherry");
    }

    #[test]
    fn test_sort_reverse_sorted() {
        let mut data = vec![
            TestString::new("cherry"),
            TestString::new("banana"),
            TestString::new("apple"),
        ];

        sort(&mut data);

        assert_eq!(data[0].data, b"apple");
        assert_eq!(data[1].data, b"banana");
        assert_eq!(data[2].data, b"cherry");
    }

    #[test]
    fn test_sort_single_element() {
        let mut data = vec![TestString::new("apple")];

        sort(&mut data);

        assert_eq!(data[0].data, b"apple");
    }

    #[test]
    fn test_sort_empty() {
        let mut data: Vec<TestString> = vec![];

        sort(&mut data);

        assert_eq!(data.len(), 0);
    }

    #[test]
    fn test_sort_large_set() {
        let mut data = vec![
            TestString::new("zebra"),
            TestString::new("apple"),
            TestString::new("mango"),
            TestString::new("banana"),
            TestString::new("orange"),
            TestString::new("grape"),
            TestString::new("kiwi"),
            TestString::new("peach"),
            TestString::new("lemon"),
            TestString::new("cherry"),
            TestString::new("date"),
            TestString::new("fig"),
        ];

        sort(&mut data);

        assert_eq!(data[0].data, b"apple");
        assert_eq!(data[1].data, b"banana");
        assert_eq!(data[2].data, b"cherry");
        assert_eq!(data[11].data, b"zebra");
    }

    #[test]
    fn test_sort_count_return() {
        let mut data = vec![
            TestString::new("apple"),
            TestString::new("apple"),
            TestString::new("banana"),
        ];

        let count = sort(&mut data);

        // Should return count of unique prefixes
        assert!(count > 0);
    }
}
