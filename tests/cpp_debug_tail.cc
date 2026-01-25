#include <marisa.h>
#include <iostream>
#include <iomanip>

int main() {
    marisa::Keyset keyset;
    keyset.push_back("a");
    keyset.push_back("app");

    marisa::Trie trie;
    trie.build(keyset);

    std::cout << "Trie built successfully" << std::endl;
    std::cout << "  num_keys: " << trie.num_keys() << std::endl;
    std::cout << "  num_nodes: " << trie.num_nodes() << std::endl;
    std::cout << std::endl;

    // Save to inspect
    trie.save("tmp/cpp_debug.marisa");

    // Test lookups with detailed agent state
    const char* words[] = {"a", "app"};
    for (int i = 0; i < 2; i++) {
        marisa::Agent agent;
        agent.set_query(words[i]);

        std::cout << "Looking up: \"" << words[i] << "\"" << std::endl;
        std::cout << "  Query length: " << agent.query().length() << std::endl;

        if (trie.lookup(agent)) {
            std::cout << "  ✓ Found (key_id=" << agent.key().id() << ")" << std::endl;
            std::cout << "  Result length: " << agent.key().length() << std::endl;
            std::cout << "  Result: \"";
            for (std::size_t j = 0; j < agent.key().length(); ++j) {
                std::cout << agent.key()[j];
            }
            std::cout << "\"" << std::endl;
        } else {
            std::cout << "  ✗ NOT FOUND" << std::endl;
        }
        std::cout << std::endl;
    }

    return 0;
}
