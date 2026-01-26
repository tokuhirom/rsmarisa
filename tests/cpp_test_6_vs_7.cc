#include <marisa.h>
#include <iostream>

void test_words(const char** words, int count, const char* filename) {
    marisa::Keyset keyset;
    for (int i = 0; i < count; i++) {
        keyset.push_back(words[i]);
    }

    marisa::Trie trie;
    trie.build(keyset);
    trie.save(filename);

    std::cout << count << " words:" << std::endl;
    std::cout << "  num_keys: " << trie.num_keys() << std::endl;
    std::cout << "  num_nodes: " << trie.num_nodes() << std::endl;
    std::cout << "  Saved to " << filename << std::endl;

    // Test lookups
    for (int i = 0; i < count; i++) {
        marisa::Agent agent;
        agent.set_query(words[i]);
        if (trie.lookup(agent)) {
            std::cout << "  ✓ " << words[i] << std::endl;
        } else {
            std::cout << "  ✗ " << words[i] << " NOT FOUND" << std::endl;
        }
    }
    std::cout << std::endl;
}

int main() {
    const char* words_6[] = {
        "a", "app", "apple", "application", "apply", "banana"
    };

    const char* words_7[] = {
        "a", "app", "apple", "application", "apply", "banana", "band"
    };

    test_words(words_6, 6, "tmp/cpp_6words.marisa");
    test_words(words_7, 7, "tmp/cpp_7words.marisa");

    return 0;
}
