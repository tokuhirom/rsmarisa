//! Debug tool to show sizes of trie components

use rsmarisa::{Keyset, Trie};

fn main() {
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

    for word in &words {
        keyset.push_back_str(word).unwrap();
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Trie component sizes:");
    println!("  Total I/O size: {} bytes", trie.io_size());
    println!("  Keys: {}", trie.num_keys());
    println!("  Nodes: {}", trie.num_nodes());
}
