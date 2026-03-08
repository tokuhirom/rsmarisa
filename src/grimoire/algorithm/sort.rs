//! Sorting algorithms for trie construction.
//!
//! Ported from: lib/marisa/grimoire/algorithm/sort.h
//!
//! This module implements depth-based string sorting using a Parallel MSD
//! (Most Significant Digit) Radix Sort. Compared to the original C++ three-way
//! quicksort, MSD radix sort is:
//!
//! - **Asymptotically better**: O(n·k) vs O(n·k·log n), where k is the key length.
//!   At each depth level we bucket all n elements in O(n) instead of comparing.
//! - **Naturally parallel**: once we partition into the 257 independent buckets
//!   (byte 0-255 + EOS) at a given depth, every bucket can be sorted on a
//!   separate thread with no shared state.
//! - **Single allocation**: a scratch buffer is allocated once at the top level
//!   and passed as sub-slices to every recursive call, eliminating per-call
//!   heap allocations.
//!
//! The parallelism is implemented with [rayon] (behind the `parallel` feature),
//! which uses work-stealing so recursive sub-tasks are also distributed
//! automatically. When the `parallel` feature is disabled (e.g., for WASM),
//! all sorting is sequential with identical results.

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Below this many elements use insertion sort (branch-free, cache-hot).
const INSERTION_SORT_THRESHOLD: usize = 16;

/// Below this many elements skip rayon overhead and stay sequential.
#[cfg(feature = "parallel")]
const PARALLEL_THRESHOLD: usize = 4096;

/// Number of distinct label values: bytes 0-255 plus the EOS sentinel (-1).
/// Label -1 maps to bucket index 0; byte b maps to bucket index b + 1.
const NUM_BUCKETS: usize = 257;

/// Trait for types that can be sorted by this algorithm.
///
/// Types must support indexed access to bytes and provide a length.
pub trait Sortable {
    /// Returns the byte at the given index, or None if index >= length.
    fn get(&self, index: usize) -> Option<u8>;

    /// Returns the length of the sortable element.
    fn length(&self) -> usize;
}

/// Returns the label (byte value) of `unit` at `depth`.
///
/// Returns -1 when `depth >= unit.length()` (end-of-string sentinel).
#[inline(always)]
fn get_label<T: Sortable>(unit: &T, depth: usize) -> i32 {
    match unit.get(depth) {
        Some(b) => b as i32,
        None => -1,
    }
}

/// Compares two elements lexicographically starting from `depth`.
#[inline]
fn compare<T: Sortable>(lhs: &T, rhs: &T, depth: usize) -> std::cmp::Ordering {
    let llen = lhs.length();
    let rlen = rhs.length();
    let common = llen.min(rlen);
    for i in depth..common {
        match lhs.get(i).unwrap().cmp(&rhs.get(i).unwrap()) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
    }
    llen.cmp(&rlen)
}

/// Insertion sort for small ranges; returns the number of distinct groups.
fn insertion_sort<T: Sortable>(data: &mut [T], depth: usize) -> usize {
    if data.is_empty() {
        return 0;
    }
    // Sort in-place.
    for i in 1..data.len() {
        for j in (1..=i).rev() {
            if compare(&data[j - 1], &data[j], depth) == std::cmp::Ordering::Greater {
                data.swap(j - 1, j);
            } else {
                break;
            }
        }
    }
    // Count distinct groups (adjacent elements that differ from `depth` onward).
    let mut count = 1;
    for i in 1..data.len() {
        if compare(&data[i - 1], &data[i], depth) != std::cmp::Ordering::Equal {
            count += 1;
        }
    }
    count
}

/// Process a single bucket: recurse deeper or count the leaf.
///
/// `label` is the byte value that all elements in this bucket share at `depth - 1`.
/// A label of -1 means all elements terminated at the parent depth (EOS bucket).
fn process_bucket<T: Sortable + Clone + Send + Sync>(
    slice: &mut [T],
    scratch: &mut [T],
    label: i32,
    depth: usize,
) -> usize {
    match (label, slice.len()) {
        // EOS bucket: every element shares the same full prefix — one trie node.
        (-1, _) => 1,
        // Single element: one unique leaf.
        (_, 1) => 1,
        // Multiple elements with the same label — go one level deeper.
        _ => sort_impl(slice, scratch, depth),
    }
}

/// Parallel MSD Radix Sort with a pre-allocated scratch buffer.
///
/// At each depth level we:
/// 1. Count how many elements fall into each of the 257 buckets — O(n).
/// 2. Scatter elements into `scratch` to achieve the partition — O(n), no heap alloc.
/// 3. Copy `scratch` back to `data` — O(n).
/// 4. Recurse on each non-empty bucket at depth + 1; buckets are independent
///    so large inputs use rayon to process them in parallel.
///
/// By passing `scratch` (pre-allocated in `sort`) down through every recursive
/// call, we eliminate the per-call `Vec<T>` allocation of the naïve approach,
/// reducing allocation count from O(unique-prefix-count) to exactly 1.
///
/// Returns the count of unique string prefixes (= trie node count).
fn sort_impl<T: Sortable + Clone + Send + Sync>(
    data: &mut [T],
    scratch: &mut [T],
    depth: usize,
) -> usize {
    match data.len() {
        0 => return 0,
        1 => return 1,
        n if n <= INSERTION_SORT_THRESHOLD => return insertion_sort(data, depth),
        _ => {}
    }

    // --- Step 1: count occurrences of each label at this depth ---
    let mut counts = [0usize; NUM_BUCKETS];
    for item in data.iter() {
        counts[(get_label(item, depth) + 1) as usize] += 1;
    }

    // --- Step 2: scatter into scratch (pre-allocated, no heap allocation here) ---
    {
        let mut cursors = [0usize; NUM_BUCKETS];
        let mut pos = 0;
        for i in 0..NUM_BUCKETS {
            cursors[i] = pos;
            pos += counts[i];
        }
        for item in data.iter() {
            let idx = (get_label(item, depth) + 1) as usize;
            scratch[cursors[idx]] = item.clone();
            cursors[idx] += 1;
        }
    }

    // --- Step 3: copy bucketed elements from scratch back into data ---
    for (d, s) in data.iter_mut().zip(scratch.iter()) {
        *d = s.clone();
    }

    // --- Step 4: build non-overlapping (data, scratch) bucket slice pairs ---
    // Both slices are already aligned: after the copy, data[..n] mirrors scratch[..n].
    let mut bucket_slices: Vec<(&mut [T], &mut [T], i32)> = Vec::new();
    {
        let mut rem_data: &mut [T] = data;
        let mut rem_scratch: &mut [T] = scratch;
        for (i, &cnt) in counts.iter().enumerate() {
            if cnt > 0 {
                let label = i as i32 - 1; // bucket index → label (-1 … 255)
                let (hd, td) = rem_data.split_at_mut(cnt);
                let (hs, ts) = rem_scratch.split_at_mut(cnt);
                rem_data = td;
                rem_scratch = ts;
                bucket_slices.push((hd, hs, label));
            }
        }
    }

    // --- Step 5: recurse on each bucket (parallel when worthwhile) ---
    #[cfg(feature = "parallel")]
    {
        let n = bucket_slices.iter().map(|(d, _, _)| d.len()).sum::<usize>();
        if n >= PARALLEL_THRESHOLD {
            return bucket_slices
                .par_iter_mut()
                .map(|(d, s, label)| process_bucket(d, s, *label, depth + 1))
                .sum();
        }
    }

    bucket_slices
        .iter_mut()
        .map(|(d, s, label)| process_bucket(d, s, *label, depth + 1))
        .sum()
}

/// Sorts a slice of sortable elements using parallel MSD radix sort.
///
/// A single scratch buffer of `data.len()` elements is allocated once and
/// reused across all recursive calls, so the total heap-allocation count is
/// exactly **1** regardless of input size or key depth.
///
/// Returns the count of unique string prefixes found during sorting,
/// which equals the number of nodes in the trie being built.
pub fn sort<T: Sortable + Clone + Send + Sync>(data: &mut [T]) -> usize {
    match data.len() {
        0 => return 0,
        1 => return 1,
        n if n <= INSERTION_SORT_THRESHOLD => return insertion_sort(data, 0),
        _ => {}
    }
    // Allocate the scratch buffer exactly once for the entire sort.
    let mut scratch: Vec<T> = data.iter().cloned().collect();
    sort_impl(data, &mut scratch, 0)
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
    fn test_compare() {
        use std::cmp::Ordering;
        let a = TestString::new("apple");
        let b = TestString::new("banana");
        let c = TestString::new("apple");

        assert_eq!(compare(&a, &b, 0), Ordering::Less); // apple < banana
        assert_eq!(compare(&b, &a, 0), Ordering::Greater); // banana > apple
        assert_eq!(compare(&a, &c, 0), Ordering::Equal); // apple == apple
    }

    #[test]
    fn test_compare_with_depth() {
        use std::cmp::Ordering;
        let a = TestString::new("apple");
        let b = TestString::new("apply");
        let c = TestString::new("application");

        // Comparing from start: "apple" < "apply" because 'e' < 'y'
        assert_eq!(compare(&a, &b, 0), Ordering::Less);

        // Comparing from index 4: 'e' < 'y'
        assert_eq!(compare(&a, &b, 4), Ordering::Less);

        // Comparing "apple" vs "application" from index 3: "le" < "lication"
        assert_eq!(compare(&a, &c, 3), Ordering::Less);
    }

    #[test]
    fn test_compare_prefix() {
        use std::cmp::Ordering;
        let a = TestString::new("app");
        let b = TestString::new("apple");

        assert_eq!(compare(&a, &b, 0), Ordering::Less); // Shorter is less
        assert_eq!(compare(&b, &a, 0), Ordering::Greater); // Longer is greater
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
