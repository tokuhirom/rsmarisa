//! Basic usage example.
//!
//! This example demonstrates basic trie operations including building,
//! searching, and querying trie statistics.

use marisa::{Agent, Keyset, Trie};

fn main() {
    println!("=== rust-marisa - Basic Usage Example ===\n");

    // Create a keyset and add some keys
    let mut keyset = Keyset::new();
    keyset.push_back_str("app").unwrap();
    keyset.push_back_str("apple").unwrap();
    keyset.push_back_str("application").unwrap();
    keyset.push_back_str("apply").unwrap();

    println!("Building trie with {} keys...", keyset.size());

    // Build the trie
    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Trie built successfully!");
    println!("  Number of keys: {}", trie.num_keys());
    println!("  Number of nodes: {}", trie.num_nodes());
    println!("  Total size: {} bytes", trie.total_size());
    println!("  I/O size: {} bytes\n", trie.io_size());

    // Lookup examples
    println!("=== Lookup Examples ===");
    let mut agent = Agent::new();
    agent.init_state().unwrap();

    let test_words = vec!["apple", "apply", "apricot", "app"];
    for word in test_words {
        agent.set_query_str(word);
        let found = trie.lookup(&mut agent);
        println!("  lookup(\"{}\") = {}", word, found);
    }

    // Common prefix search example
    println!("\n=== Common Prefix Search ===");
    agent.set_query_str("application");
    println!("Finding all prefixes of \"application\":");
    while trie.common_prefix_search(&mut agent) {
        println!(
            "  Found: \"{}\" (id={})",
            agent.key().as_str(),
            agent.key().id()
        );
    }

    println!("\n=== Example Complete ===");
}
