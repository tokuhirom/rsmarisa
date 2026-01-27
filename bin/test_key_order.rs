use rsmarisa::{Agent, Keyset, Trie};

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

    // Build trie
    let mut keyset = Keyset::new();
    for word in &words {
        keyset.push_back_str(word).unwrap();
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Trie num_keys: {}", trie.num_keys());
    println!();

    // Check lookup for each word
    println!("Lookup results:");
    for (i, word) in words.iter().enumerate() {
        let mut agent = Agent::new();
        agent.set_query_str(word);
        if trie.lookup(&mut agent) {
            println!("  {:2}: {} -> key_id={}", i, word, agent.key().id());
        } else {
            println!("  {:2}: {} -> NOT FOUND", i, word);
        }
    }
    println!();

    // Reverse lookup
    println!("Reverse lookup by key_id:");
    for key_id in 0..trie.num_keys() {
        let mut agent = Agent::new();
        agent.set_query_id(key_id);
        trie.reverse_lookup(&mut agent);
        let key_str = std::str::from_utf8(agent.key().as_bytes()).unwrap();
        println!("  key_id={:2}: {}", key_id, key_str);
    }
}
