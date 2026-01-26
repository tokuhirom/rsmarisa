use marisa::agent::Agent;
use marisa::keyset::Keyset;
use marisa::trie::Trie;

fn main() {
    let all_words = vec![
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

    // Try adding words one by one
    for n in 1..=15 {
        let words = &all_words[..n];

        let mut keyset = Keyset::new();
        for word in words {
            let _ = keyset.push_back_str(word);
        }

        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);

        // Test all words in this set
        let mut all_found = true;
        for word in words {
            let mut agent = Agent::new();
            agent.set_query_str(word);

            if !trie.lookup(&mut agent) {
                all_found = false;
                println!("✗ With {} words, '{}' NOT FOUND", n, word);
            }
        }

        if all_found {
            println!("✓ All {} words found", n);
        }
    }
}
