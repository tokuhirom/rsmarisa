use marisa::{Agent, Keyset, Trie};

fn main() {
    let mut keyset = Keyset::new();
    keyset.push_back_str("a").unwrap();
    keyset.push_back_str("app").unwrap();

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Built trie with [\"a\", \"app\"]");
    println!("  num_keys: {}", trie.num_keys());
    println!("  num_nodes: {}", trie.num_nodes());
    println!();

    // Save to file for analysis BEFORE testing lookup
    trie.save("tmp/rust_a_app.marisa").expect("Failed to save");
    println!("Saved to tmp/rust_a_app.marisa");
    println!();

    // Test lookup
    let words = vec!["a", "app"];
    for word in &words {
        let mut agent = Agent::new();
        agent.set_query_str(word);

        if trie.lookup(&mut agent) {
            println!("✓ Found: {} (key_id={})", word, agent.key().id());
        } else {
            println!("✗ NOT FOUND: {}", word);
        }
    }
}
