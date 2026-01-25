/// Test to verify that Rust marisa produces the same key_id assignments as C++ marisa-trie
///
/// This test verifies that when building a trie with the same test data,
/// both implementations assign the same key_id to each word.

#[test]
fn test_key_id_assignment_matches_cpp() {
    use marisa::{Agent, Keyset, Trie};

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

    // Expected key_id for each word based on C++ marisa-trie output
    // From: LD_LIBRARY_PATH=.../lib/.libs tmp/test_sort_cpp
    // Reverse lookup by key_id:
    //   key_id=0: a
    //   key_id=1: app
    //   key_id=2: banana
    //   key_id=3: band
    //   key_id=4: bank
    //   key_id=5: test
    //   key_id=6: trie
    //   key_id=7: can
    //   key_id=8: cat
    //   key_id=9: dog
    //   key_id=10: door
    //   key_id=11: testing
    //   key_id=12: apple
    //   key_id=13: application
    //   key_id=14: apply
    let expected_key_ids = vec![
        ("a", 0),
        ("app", 1),
        ("apple", 12),
        ("application", 13),
        ("apply", 14),
        ("banana", 2),
        ("band", 3),
        ("bank", 4),
        ("can", 7),
        ("cat", 8),
        ("dog", 9),
        ("door", 10),
        ("test", 5),
        ("testing", 11),
        ("trie", 6),
    ];

    // Verify each word has the expected key_id
    for (word, expected_id) in &expected_key_ids {
        let mut agent = Agent::new();
        agent.set_query_str(word);

        assert!(
            trie.lookup(&mut agent),
            "Word '{}' should be found in trie",
            word
        );

        let actual_id = agent.key().id();
        assert_eq!(
            actual_id, *expected_id,
            "Word '{}' has key_id={}, expected {}",
            word, actual_id, expected_id
        );
    }

    // Verify reverse lookup produces the expected order
    let expected_reverse_order = vec![
        "a", "app", "banana", "band", "bank", "test", "trie",
        "can", "cat", "dog", "door", "testing", "apple", "application", "apply",
    ];

    for (key_id, expected_word) in expected_reverse_order.iter().enumerate() {
        let mut agent = Agent::new();
        agent.set_query_id(key_id);
        trie.reverse_lookup(&mut agent);

        let actual_word = std::str::from_utf8(agent.key().as_bytes()).unwrap();
        assert_eq!(
            actual_word, *expected_word,
            "Reverse lookup at key_id={} returned '{}', expected '{}'",
            key_id, actual_word, expected_word
        );
    }
}
