use rsmarisa::{Agent, Keyset, Trie};

fn main() {
    // Build a simple trie
    let keys = vec![
        "a",
        "app",
        "apple",
        "application",
        "apply",
        "banana",
        "band",
    ];
    let mut keyset = Keyset::new();
    for key in &keys {
        keyset.push_back_str(key).unwrap();
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Built trie with {} keys", trie.num_keys());

    // First, get the ID for each key via lookup
    let mut agent = Agent::new();
    let mut key_to_id = std::collections::HashMap::new();

    for key in &keys {
        agent.set_query_str(key);
        if trie.lookup(&mut agent) {
            let id = agent.key().id();
            key_to_id.insert(*key, id);
            println!("Key '{}' has ID {}", key, id);
        }
    }

    println!();

    // Now test reverse lookup for each ID
    let mut all_ok = true;
    for (key, &id) in &key_to_id {
        agent.set_query_id(id);
        trie.reverse_lookup(&mut agent);

        let restored_bytes = agent.key().as_bytes();

        match std::str::from_utf8(restored_bytes) {
            Ok(restored_key) => {
                if restored_key != *key {
                    println!(
                        "ID {}: ✗ MISMATCH! expected '{}', got '{}'",
                        id, key, restored_key
                    );
                    all_ok = false;
                } else {
                    println!("ID {}: ✓ OK '{}'", id, restored_key);
                }
            }
            Err(e) => {
                println!("ID {}: ✗ Invalid UTF-8: {:?}", id, e);
                all_ok = false;
            }
        }
    }

    println!();
    if all_ok {
        println!("✓ All reverse lookups successful!");
    } else {
        println!("✗ Some reverse lookups failed");
        std::process::exit(1);
    }
}
