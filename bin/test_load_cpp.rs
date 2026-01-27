use rsmarisa::{Agent, Trie};

fn main() {
    // Load C++ generated file
    let mut trie = Trie::new();
    trie.load("tmp/cpp_a_app.marisa").expect("Failed to load");

    println!("Loaded C++ generated trie");
    println!("  num_keys: {}", trie.num_keys());
    println!("  num_nodes: {}", trie.num_nodes());
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
