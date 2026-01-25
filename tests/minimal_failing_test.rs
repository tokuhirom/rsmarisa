/// Minimal test to isolate the failing case

#[test]
fn test_two_words_app_apple() {
    use marisa::{Agent, Keyset, Trie};

    let words = vec!["app", "apple"];

    let mut keyset = Keyset::new();
    for word in &words {
        keyset.push_back_str(word).unwrap();
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Test: {:?}", words);
    for word in &words {
        let mut agent = Agent::new();
        agent.set_query_str(word);
        assert!(trie.lookup(&mut agent), "Should find '{}'", word);
        println!("  ✓ Found: {}", word);
    }
}

#[test]
fn test_three_words_a_app_apple() {
    use marisa::{Agent, Keyset, Trie};

    let words = vec!["a", "app", "apple"];

    let mut keyset = Keyset::new();
    for word in &words {
        keyset.push_back_str(word).unwrap();
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Test: {:?}", words);
    for word in &words {
        let mut agent = Agent::new();
        agent.set_query_str(word);
        assert!(trie.lookup(&mut agent), "Should find '{}'", word);
        println!("  ✓ Found: {}", word);
    }
}

#[test]
fn test_two_words_a_app() {
    use marisa::{Agent, Keyset, Trie};

    let words = vec!["a", "app"];

    let mut keyset = Keyset::new();
    for word in &words {
        keyset.push_back_str(word).unwrap();
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Test: {:?}", words);
    for word in &words {
        let mut agent = Agent::new();
        agent.set_query_str(word);
        assert!(trie.lookup(&mut agent), "Should find '{}'", word);
        println!("  ✓ Found: {}", word);
    }
}

#[test]
fn test_two_words_a_apple() {
    use marisa::{Agent, Keyset, Trie};

    let words = vec!["a", "apple"];

    let mut keyset = Keyset::new();
    for word in &words {
        keyset.push_back_str(word).unwrap();
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Test: {:?}", words);
    for word in &words {
        let mut agent = Agent::new();
        agent.set_query_str(word);
        assert!(trie.lookup(&mut agent), "Should find '{}'", word);
        println!("  ✓ Found: {}", word);
    }
}
