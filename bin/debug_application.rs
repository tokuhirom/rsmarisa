use rsmarisa::agent::Agent;
use rsmarisa::keyset::Keyset;
use rsmarisa::trie::Trie;

fn main() {
    // Test the specific failing words in different combinations
    let test_cases = [
        &["application"][..],
        &["app", "application"][..],
        &["a", "app", "apple", "application"][..],
        &["a", "app", "apple", "application", "apply"][..], // First 5 words
    ];

    for (i, words) in test_cases.iter().enumerate() {
        println!("\n=== Test {}: {:?} ===", i + 1, words);

        let mut keyset = Keyset::new();
        for word in *words {
            let _ = keyset.push_back_str(word);
        }

        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);

        println!(
            "Built trie: {} keys, {} nodes",
            trie.num_keys(),
            trie.num_nodes()
        );

        for word in *words {
            let mut agent = Agent::new();
            agent.set_query_str(word);

            if trie.lookup(&mut agent) {
                println!("  ✓ {}", word);
            } else {
                println!("  ✗ {} NOT FOUND", word);
            }
        }
    }
}
