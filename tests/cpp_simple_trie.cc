#include <marisa.h>
#include <iostream>

int main() {
    // Very simple test
    marisa::Keyset keyset;
    keyset.push_back("a");
    keyset.push_back("app");

    marisa::Trie trie;
    trie.build(keyset);
    trie.save("tmp/cpp_simple.marisa");

    std::cout << "Built simple trie: " << trie.num_keys() << " keys" << std::endl;

    // Test lookups
    marisa::Agent agent;
    agent.set_query("a");
    std::cout << "a: " << (trie.lookup(agent) ? "found" : "NOT FOUND") << std::endl;

    agent.set_query("app");
    std::cout << "app: " << (trie.lookup(agent) ? "found" : "NOT FOUND") << std::endl;

    return 0;
}
