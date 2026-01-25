use marisa::grimoire::algorithm::sort;
use marisa::grimoire::trie::entry::Entry;

fn main() {
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

    // Create entries exactly like in tail.rs
    let mut entries: Vec<Entry> = Vec::new();
    for (id, word) in words.iter().enumerate() {
        let mut entry = Entry::new();
        entry.set_str(word.as_bytes());
        entry.set_id(id);
        entries.push(entry);
    }

    // Test Entry::get() for a few words to verify reverse indexing
    println!("Entry::get() reverse indexing test:");
    for i in 0..3 {
        let word = words[i];
        let entry = &entries[i];
        print!("  '{}' -> ", word);
        for j in 0..entry.length() {
            print!("{:02x} ", entry.get(j));
        }
        println!();
    }
    println!();

    // Sort using the algorithm
    let num_sorted = sort::sort(&mut entries[..]);

    println!("After sorting ({} unique):", num_sorted);
    for (i, entry) in entries.iter().enumerate() {
        let word = std::str::from_utf8(entry.as_bytes()).unwrap();
        println!("  {:2}: {} (id={})", i, word, entry.id());
    }
    println!();

    // Expected C++ order (from test_sort_cpp output):
    // key_id=0: a
    // key_id=1: app
    // key_id=2: banana
    // key_id=3: band
    // key_id=4: bank
    // key_id=5: test
    // key_id=6: trie
    // key_id=7: can
    // key_id=8: cat
    // key_id=9: dog
    // key_id=10: door
    // key_id=11: testing
    // key_id=12: apple
    // key_id=13: application
    // key_id=14: apply

    println!("Expected order (from C++ marisa-trie):");
    let expected = vec![
        "a", "app", "banana", "band", "bank", "test", "trie",
        "can", "cat", "dog", "door", "testing", "apple", "application", "apply"
    ];
    for (i, word) in expected.iter().enumerate() {
        println!("  {:2}: {}", i, word);
    }
}
