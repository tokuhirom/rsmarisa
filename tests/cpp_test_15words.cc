#include <marisa.h>
#include <iostream>

int main() {
    marisa::Keyset keyset;

    const char *words[] = {
        "a", "app", "apple", "application", "apply",
        "banana", "band", "bank", "can", "cat",
        "dog", "door", "test", "testing", "trie",
    };

    for (int i = 0; i < 15; i++) {
        keyset.push_back(words[i]);
    }

    marisa::Trie trie;
    trie.build(keyset);

    std::cout << "Built trie with " << trie.num_keys() << " keys\n" << std::endl;

    for (int i = 0; i < 15; i++) {
        marisa::Agent agent;
        agent.set_query(words[i]);

        if (trie.lookup(agent)) {
            std::cout << "✓ Found: " << words[i] << std::endl;
        } else {
            std::cout << "✗ NOT FOUND: " << words[i] << std::endl;
        }
    }

    return 0;
}
