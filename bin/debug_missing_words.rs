use marisa::keyset::Keyset;
use marisa::trie::Trie;
use marisa::agent::Agent;

fn main() {
    // Test just the failing words
    let test_sets = vec![
        vec!["application"],
        vec!["banana"],
        vec!["band"],
        vec!["bank"],
        vec!["app", "application"],  // Test with prefix
        vec!["ban", "banana", "band", "bank"],  // Test all ban* words
    ];

    for (i, words) in test_sets.iter().enumerate() {
        println!("\n=== Test set {}: {:?} ===", i + 1, words);

        let mut keyset = Keyset::new();
        for word in words {
            keyset.push_back_str(word);
        }

        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);

        println!("Built trie with {} keys", trie.num_keys());

        for word in words {
            let mut agent = Agent::new();
            agent.set_query_str(word);

            if trie.lookup(&mut agent) {
                println!("  ✓ Found: {}", word);
            } else {
                println!("  ✗ NOT FOUND: {}", word);
            }
        }
    }
}
