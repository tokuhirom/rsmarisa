/// Test that Key sorting produces the same order as C++ marisa-trie

#[test]
fn test_key_sort_order() {
    use marisa::grimoire::algorithm::sort;
    use marisa::grimoire::trie::key::Key;
    use marisa::grimoire::vector::vector::Vector;

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

    // Create Keys like C++ does
    let mut keys: Vector<Key> = Vector::new();
    keys.resize(words.len(), Key::new());
    for (i, word) in words.iter().enumerate() {
        keys[i].set_str(word.as_bytes());
        keys[i].set_id(i);
    }

    // Sort
    let keys_slice = keys.as_mut_slice();
    let num_unique = sort::sort(keys_slice);

    println!("After sorting ({} unique):", num_unique);
    for (i, key) in keys_slice.iter().enumerate() {
        let word = std::str::from_utf8(key.as_bytes()).unwrap();
        println!("  {:2}: {} (id={})", i, word, key.id());
    }
    println!();

    // Expected order from C++ (lexicographically sorted forward strings):
    // a, app, apple, application, apply, banana, band, bank, can, cat, dog, door, test, testing, trie
    let expected_sorted_order = vec![
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

    println!("Expected order (lexicographic):");
    for (i, word) in expected_sorted_order.iter().enumerate() {
        println!("  {:2}: {}", i, word);
    }
    println!();

    // Verify
    for (i, expected_word) in expected_sorted_order.iter().enumerate() {
        let actual_word = std::str::from_utf8(keys_slice[i].as_bytes()).unwrap();
        assert_eq!(
            actual_word, *expected_word,
            "Position {}: expected '{}', got '{}'",
            i, expected_word, actual_word
        );
    }

    println!("✓ Key sorting matches expected lexicographic order");
}
