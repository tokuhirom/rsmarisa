use marisa::agent::Agent;
use marisa::keyset::Keyset;
use marisa::trie::Trie;

fn main() {
    let words = vec![
        "a",
        "app",
        "apple",
        "application",
        "apply",
        "banana",
        "band",
    ];

    // Build Rust trie
    let mut keyset = Keyset::new();
    for word in &words {
        let _ = keyset.push_back_str(word);
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("=== Rust-generated trie ===");
    println!(
        "num_keys: {}, num_nodes: {}\n",
        trie.num_keys(),
        trie.num_nodes()
    );

    for word in &words {
        let mut agent = Agent::new();
        agent.set_query_str(word);

        if trie.lookup(&mut agent) {
            println!("✓ {}", word);
        } else {
            println!("✗ {} NOT FOUND", word);
        }
    }

    println!("\n=== Loading C++-generated trie ===");

    let mut cpp_trie = Trie::new();
    cpp_trie
        .load("tmp/cpp_7words.marisa")
        .expect("Failed to load");

    println!(
        "num_keys: {}, num_nodes: {}\n",
        cpp_trie.num_keys(),
        cpp_trie.num_nodes()
    );

    for word in &words {
        let mut agent = Agent::new();
        agent.set_query_str(word);

        if cpp_trie.lookup(&mut agent) {
            println!("✓ {}", word);
        } else {
            println!("✗ {} NOT FOUND", word);
        }
    }
}
