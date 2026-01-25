#include <marisa.h>
#include <iostream>

int main() {
    // Load Rust-generated binary
    marisa::Trie trie;

    try {
        trie.load("tmp/rust_a_app.marisa");

        std::cout << "Loaded Rust-generated trie from tmp/rust_a_app.marisa" << std::endl;
        std::cout << "  num_keys: " << trie.num_keys() << std::endl;
        std::cout << "  num_nodes: " << trie.num_nodes() << std::endl;
        std::cout << std::endl;

        // Test lookup
        const char* words[] = {"a", "app"};
        for (int i = 0; i < 2; i++) {
            marisa::Agent agent;
            agent.set_query(words[i]);

            if (trie.lookup(agent)) {
                std::cout << "✓ Found: " << words[i] << " (key_id=" << agent.key().id() << ")" << std::endl;
            } else {
                std::cout << "✗ NOT FOUND: " << words[i] << std::endl;
            }
        }
    } catch (const marisa::Exception &ex) {
        std::cerr << "Error: " << ex.what() << std::endl;
        return 1;
    }

    return 0;
}
