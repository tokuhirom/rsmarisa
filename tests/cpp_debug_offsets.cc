#include <marisa.h>
#include <iostream>
#include <iomanip>

int main() {
    marisa::Keyset keyset;
    keyset.push_back("a");
    keyset.push_back("app");

    marisa::Trie trie;
    trie.build(keyset);

    std::cout << "Trie built with [\"a\", \"app\"]" << std::endl;
    std::cout << "  num_keys: " << trie.num_keys() << std::endl;
    std::cout << std::endl;

    // Test lookup with detailed tracing
    std::cout << "=== Lookup Test ===" << std::endl;

    const char* words[] = {"a", "app"};
    for (int i = 0; i < 2; i++) {
        marisa::Agent agent;
        agent.set_query(words[i]);

        std::cout << "\nLooking up: \"" << words[i] << "\"" << std::endl;

        if (trie.lookup(agent)) {
            std::cout << "  ✓ Found (key_id=" << agent.key().id() << ")" << std::endl;
        } else {
            std::cout << "  ✗ NOT FOUND" << std::endl;
        }
    }

    return 0;
}
