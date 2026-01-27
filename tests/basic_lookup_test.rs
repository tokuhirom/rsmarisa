/// Basic test to verify all inserted words can be found

#[test]
fn test_all_words_can_be_found() {
    use rsmarisa::{Agent, Keyset, Trie};

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

    // Build trie
    let mut keyset = Keyset::new();
    for word in &words {
        keyset.push_back_str(word).unwrap();
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Trie built successfully:");
    println!("  num_keys: {}", trie.num_keys());
    println!("  num_nodes: {}", trie.num_nodes());
    println!();

    // Try to lookup each word
    let mut found = Vec::new();
    let mut not_found = Vec::new();

    for word in &words {
        let mut agent = Agent::new();
        agent.set_query_str(word);

        if trie.lookup(&mut agent) {
            found.push(*word);
            println!("✓ Found: {}", word);
        } else {
            not_found.push(*word);
            println!("✗ NOT FOUND: {}", word);
        }
    }

    println!();
    println!("Summary:");
    println!("  Found: {} / {}", found.len(), words.len());
    println!("  Not found: {}", not_found.len());

    if !not_found.is_empty() {
        println!();
        println!("Missing words:");
        for word in &not_found {
            println!("  - {}", word);
        }
        panic!("{} words are missing from the trie", not_found.len());
    }
}
