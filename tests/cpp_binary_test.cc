#include <marisa.h>
#include <iostream>
#include <iomanip>

int main() {
    marisa::Keyset keyset;

    // Test case that fails in Rust
    keyset.push_back("a");
    keyset.push_back("app");

    marisa::Trie trie;
    trie.build(keyset);

    std::cout << "Built trie with [\"a\", \"app\"]" << std::endl;
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
    std::cout << std::endl;

    // Save to file for analysis
    trie.save("tmp/cpp_a_app.marisa");
    std::cout << "Saved to tmp/cpp_a_app.marisa" << std::endl;

    return 0;
}
