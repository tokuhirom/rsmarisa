//! Test Entry sorting behavior

use rsmarisa::grimoire::algorithm::sort;
use rsmarisa::grimoire::trie::entry::Entry;

fn main() {
    // Test data - same as in the test files
    let words = vec![
        "a",
        "app",
        "apple",
        "application",
        "apply",
        "banana",
        "band",
        "bank",
        "can",
        "cat",
        "dog",
        "door",
        "test",
        "testing",
        "trie",
    ];

    println!("Original order:");
    for (i, word) in words.iter().enumerate() {
        println!("  {:2}: {}", i, word);
    }
    println!();

    // Create entries
    let mut entries: Vec<Entry> = Vec::new();
    for (id, word) in words.iter().enumerate() {
        let mut entry = Entry::new();
        entry.set_str(word.as_bytes());
        entry.set_id(id);
        entries.push(entry);
    }

    // Sort using the algorithm
    let entries_slice = &mut entries[..];
    let num_sorted = sort::sort(entries_slice);

    println!("After sorting ({} unique):", num_sorted);
    for (i, entry) in entries.iter().enumerate() {
        let word = std::str::from_utf8(entry.as_bytes()).unwrap();
        println!("  {:2}: {} (id={})", i, word, entry.id());
    }
    println!();

    // Show access order (reverse)
    println!("Reverse access pattern for each entry:");
    for entry in entries.iter().take(5) {
        let word = std::str::from_utf8(entry.as_bytes()).unwrap();
        print!("  {}: '{}' -> ", word, word);
        for j in 0..entry.length() {
            print!("{:02x} ", entry.get(j));
        }
        println!();
    }
}
