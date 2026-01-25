use marisa::keyset::Keyset;
use marisa::trie::Trie;

fn main() {
    let words = vec![
        "a", "app", "apple", "application", "apply",
        "banana", "band", "bank", "can", "cat",
        "dog", "door", "test", "testing", "trie",
    ];

    // Build Rust trie
    let mut keyset = Keyset::new();
    for word in &words {
        keyset.push_back_str(word).unwrap();
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Built Rust trie:");
    println!("  num_keys: {}", trie.num_keys());

    // Test all lookups
    for (i, word) in words.iter().enumerate() {
        let mut agent = marisa::agent::Agent::new();
        agent.set_query_str(word);

        if trie.lookup(&mut agent) {
            let key_id = agent.key().id();
            if key_id != i {
                println!("✗ {}: found with wrong id {} (expected {})", word, key_id, i);
            } else {
                println!("✓ {}: found (key_id={})", word, key_id);
            }
        } else {
            println!("✗ {}: NOT FOUND", word);
        }
    }

    println!("\n--- Testing C++-generated binary ---");

    // Load C++ binary
    let mut cpp_trie = Trie::new();
    cpp_trie.load("tmp/cpp_15words.marisa").expect("Failed to load C++ binary");

    println!("Loaded C++ trie:");
    println!("  num_keys: {}", cpp_trie.num_keys());

    // Test all lookups
    for (i, word) in words.iter().enumerate() {
        let mut agent = marisa::agent::Agent::new();
        agent.set_query_str(word);

        if cpp_trie.lookup(&mut agent) {
            let key_id = agent.key().id();
            if key_id != i {
                println!("✗ {}: found with wrong id {} (expected {})", word, key_id, i);
            } else {
                println!("✓ {}: found (key_id={})", word, key_id);
            }
        } else {
            println!("✗ {}: NOT FOUND", word);
        }
    }
}
