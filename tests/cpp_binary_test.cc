// C++ binary compatibility test
// This creates a binary file using the original marisa-trie library

#include <marisa.h>
#include <iostream>
#include <fstream>

int main(int argc, char *argv[]) {
    if (argc != 2) {
        std::cerr << "Usage: " << argv[0] << " <output_file>" << std::endl;
        return 1;
    }

    const char *output_file = argv[1];

    // Create a keyset with the same test data as Rust version
    marisa::Keyset keyset;
    const char *words[] = {
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
    };

    int num_words = sizeof(words) / sizeof(words[0]);
    std::cout << "Creating trie with " << num_words << " words" << std::endl;

    for (int i = 0; i < num_words; i++) {
        keyset.push_back(words[i]);
    }

    // Build trie
    marisa::Trie trie;
    trie.build(keyset);

    std::cout << "Trie stats:" << std::endl;
    std::cout << "  Keys: " << trie.num_keys() << std::endl;
    std::cout << "  Nodes: " << trie.num_nodes() << std::endl;
    std::cout << "  I/O size: " << trie.io_size() << " bytes" << std::endl;

    // Save to file
    trie.save(output_file);

    // Check file size
    std::ifstream file(output_file, std::ios::binary | std::ios::ate);
    std::streamsize size = file.tellg();
    file.close();

    std::cout << "Saved to '" << output_file << "': " << size << " bytes" << std::endl;

    return 0;
}
