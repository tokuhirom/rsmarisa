#include <marisa.h>
#include <iostream>

int main() {
    // First, create and save
    marisa::Keyset keyset;
    keyset.push_back("a");
    keyset.push_back("app");

    marisa::Trie trie;
    trie.build(keyset);
    trie.save("tmp/cpp_a_app.marisa");

    std::cout << "Saved trie to tmp/cpp_a_app.marisa" << std::endl;
    std::cout << "  num_keys: " << trie.num_keys() << std::endl;
    std::cout << "  num_nodes: " << trie.num_nodes() << std::endl;
    std::cout << std::endl;

    // Now load it back
    marisa::Trie loaded_trie;
    loaded_trie.load("tmp/cpp_a_app.marisa");

    std::cout << "Loaded trie from tmp/cpp_a_app.marisa" << std::endl;
    std::cout << "  num_keys: " << loaded_trie.num_keys() << std::endl;
    std::cout << "  num_nodes: " << loaded_trie.num_nodes() << std::endl;
    std::cout << std::endl;

    // Test lookup on loaded trie
    const char* words[] = {"a", "app"};
    for (int i = 0; i < 2; i++) {
        marisa::Agent agent;
        agent.set_query(words[i]);

        if (loaded_trie.lookup(agent)) {
            std::cout << "✓ Found: " << words[i] << " (key_id=" << agent.key().id() << ")" << std::endl;
        } else {
            std::cout << "✗ NOT FOUND: " << words[i] << std::endl;
        }
    }

    return 0;
}
