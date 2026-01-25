//! Binary compatibility test
//!
//! This test creates a binary file that can be compared with C++ marisa-trie output

use marisa::{Keyset, Trie};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <output_file>", args[0]);
        std::process::exit(1);
    }

    let output_file = &args[1];

    // Create a keyset with test data
    let mut keyset = Keyset::new();
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

    println!("Creating trie with {} words", words.len());
    for word in &words {
        keyset.push_back_str(word).unwrap();
    }

    // Build trie
    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Trie stats:");
    println!("  Keys: {}", trie.num_keys());
    println!("  Nodes: {}", trie.num_nodes());
    println!("  I/O size: {} bytes", trie.io_size());

    // Save to file
    trie.save(output_file).expect("Failed to save trie");

    let metadata = fs::metadata(output_file).expect("Failed to get file metadata");
    println!("Saved to '{}': {} bytes", output_file, metadata.len());
}
