use marisa::agent::Agent;
use marisa::keyset::Keyset;
use marisa::trie::Trie;

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

    // Build trie
    let mut keyset = Keyset::new();
    for word in &words {
        let _ = keyset.push_back_str(word);
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Built trie with {} keys\n", trie.num_keys());

    // Test each word
    for word in &words {
        let mut agent = Agent::new();
        agent.set_query_str(word);

        if trie.lookup(&mut agent) {
            println!("✓ Found: {}", word);
        } else {
            println!("✗ NOT FOUND: {}", word);
        }
    }
}
