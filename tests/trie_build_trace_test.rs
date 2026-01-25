/// Test to trace through trie building and compare with expected C++ behavior
/// This helps identify where the implementation diverges

#[test]
fn test_simple_trie_structure() {
    use marisa::{Agent, Keyset, Trie};

    // Start with a minimal set of words that should all be findable
    let words = vec!["a", "app", "apple"];

    let mut keyset = Keyset::new();
    for word in &words {
        keyset.push_back_str(word).unwrap();
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Built trie with words: {:?}", words);
    println!("  num_keys: {}", trie.num_keys());
    println!("  num_nodes: {}", trie.num_nodes());
    println!();

    // All words should be findable
    for word in &words {
        let mut agent = Agent::new();
        agent.set_query_str(word);

        assert!(
            trie.lookup(&mut agent),
            "Word '{}' should be findable in trie",
            word
        );
        println!("✓ Found: {} (key_id={})", word, agent.key().id());
    }

    // Test reverse lookup
    println!();
    println!("Reverse lookup:");
    for key_id in 0..trie.num_keys() {
        let mut agent = Agent::new();
        agent.set_query_id(key_id);
        trie.reverse_lookup(&mut agent);
        let word = std::str::from_utf8(agent.key().as_bytes()).unwrap();
        println!("  key_id={}: {}", key_id, word);
    }
}

#[test]
fn test_problematic_words() {
    use marisa::{Agent, Keyset, Trie};

    // Test the words that were not being found
    let words = vec!["application", "banana", "band", "bank"];

    let mut keyset = Keyset::new();
    for word in &words {
        keyset.push_back_str(word).unwrap();
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Built trie with problematic words: {:?}", words);
    println!("  num_keys: {}", trie.num_keys());
    println!("  num_nodes: {}", trie.num_nodes());
    println!();

    // All words should be findable
    for word in &words {
        let mut agent = Agent::new();
        agent.set_query_str(word);

        assert!(
            trie.lookup(&mut agent),
            "Word '{}' should be findable in trie",
            word
        );
        println!("✓ Found: {} (key_id={})", word, agent.key().id());
    }
}

#[test]
fn test_pairs_that_share_prefix() {
    use marisa::{Agent, Keyset, Trie};

    // Test words that share prefixes
    let test_cases = vec![
        vec!["app", "apple"],
        vec!["test", "testing"],
        vec!["ban", "banana", "band", "bank"],
    ];

    for words in test_cases {
        println!("\nTesting prefix group: {:?}", words);

        let mut keyset = Keyset::new();
        for word in &words {
            keyset.push_back_str(word).unwrap();
        }

        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);

        // All words should be findable
        for word in &words {
            let mut agent = Agent::new();
            agent.set_query_str(word);

            assert!(
                trie.lookup(&mut agent),
                "Word '{}' should be findable in trie (words: {:?})",
                word,
                words
            );
            println!("  ✓ Found: {}", word);
        }
    }
}
