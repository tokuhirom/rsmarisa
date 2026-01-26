use marisa::keyset::Keyset;
use marisa::trie::Trie;

fn main() {
    // Test 6 words (works)
    let words_6 = vec![
        "a", "app", "apple", "application", "apply", "banana"
    ];

    let mut keyset = Keyset::new();
    for word in &words_6 {
        keyset.push_back_str(word);
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);
    trie.save("tmp/rust_6words.marisa").expect("Failed to save");

    println!("6 words:");
    println!("  num_keys: {}", trie.num_keys());
    println!("  num_nodes: {}", trie.num_nodes());
    println!("  Saved to tmp/rust_6words.marisa");

    // Test 7 words (fails)
    let words_7 = vec![
        "a", "app", "apple", "application", "apply", "banana", "band"
    ];

    let mut keyset = Keyset::new();
    for word in &words_7 {
        keyset.push_back_str(word);
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);
    trie.save("tmp/rust_7words.marisa").expect("Failed to save");

    println!("\n7 words:");
    println!("  num_keys: {}", trie.num_keys());
    println!("  num_nodes: {}", trie.num_nodes());
    println!("  Saved to tmp/rust_7words.marisa");
}
