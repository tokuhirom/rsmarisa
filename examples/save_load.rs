//! Save and load example.
//!
//! This example demonstrates how to save a trie to a file and load it back,
//! showing the binary serialization capabilities of rust-marisa.

use rsmarisa::{Agent, Keyset, Trie};
use std::fs;

fn main() {
    println!("=== rust-marisa - Save/Load Example ===\n");

    // Build a trie with dictionary words (using same prefix for now)
    let mut keyset = Keyset::new();
    let words = vec!["app", "apple", "application", "apply", "append"];

    println!("Adding {} words to keyset...", words.len());
    for word in &words {
        keyset.push_back_str(word).unwrap();
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Trie built:");
    println!("  Keys: {}", trie.num_keys());
    println!("  Nodes: {}", trie.num_nodes());
    println!("  I/O size: {} bytes\n", trie.io_size());

    // Save to file
    let filename = "example_trie.marisa";
    println!("Saving trie to '{}'...", filename);
    trie.save(filename).expect("Failed to save trie");

    // Check file size
    let metadata = fs::metadata(filename).expect("Failed to get file metadata");
    println!("File saved: {} bytes\n", metadata.len());

    // Load from file
    println!("Loading trie from '{}'...", filename);
    let mut loaded_trie = Trie::new();
    loaded_trie.load(filename).expect("Failed to load trie");

    println!("Trie loaded:");
    println!("  Keys: {}", loaded_trie.num_keys());
    println!("  Nodes: {}", loaded_trie.num_nodes());
    println!("  I/O size: {} bytes\n", loaded_trie.io_size());

    // Verify all words can be found in loaded trie
    println!("Verifying all words can be found...");
    let mut agent = Agent::new();
    agent.init_state().unwrap();

    let mut found_count = 0;
    for word in &words {
        agent.set_query_str(word);
        if loaded_trie.lookup(&mut agent) {
            found_count += 1;
        } else {
            println!("  ERROR: Could not find '{}'", word);
        }
    }

    println!("Successfully found {}/{} words", found_count, words.len());

    if found_count == words.len() {
        println!("\n✓ All words verified successfully!");
    } else {
        println!("\n✗ Some words were not found");
    }

    // Clean up
    println!("\nCleaning up...");
    fs::remove_file(filename).expect("Failed to remove file");
    println!("File '{}' removed", filename);

    println!("\n=== Example Complete ===");
}
