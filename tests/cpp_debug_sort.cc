#include <marisa.h>
#include <iostream>
#include <marisa/grimoire/trie/entry.h>
#include <marisa/grimoire/vector.h>
#include <marisa/grimoire/algorithm/sort.h>

using namespace marisa;
using namespace marisa::grimoire;

int main() {
    const char* words[] = {
        "a", "app", "apple", "application", "apply", "banana", "band"
    };

    Vector<trie::Entry> entries;

    for (int i = 0; i < 7; i++) {
        trie::Entry e;
        e.set_str(words[i], strlen(words[i]));
        entries.push_back(e);
    }

    std::cout << "Before sort:" << std::endl;
    for (std::size_t i = 0; i < entries.size(); i++) {
        std::cout << "  " << words[i] << " (" << entries[i].length() << " chars): ";
        for (std::size_t j = 0; j < entries[i].length(); j++) {
            std::cout << entries[i][j];
        }
        std::cout << std::endl;
    }

    // Set IDs
    for (std::size_t i = 0; i < entries.size(); i++) {
        entries[i].set_id(i);
    }

    // Sort
    algorithm::sort(entries.begin(), entries.end());

    std::cout << "\nAfter sort:" << std::endl;
    for (std::size_t i = 0; i < entries.size(); i++) {
        std::cout << "  " << entries[i].ptr() << " (id=" << entries[i].id()
                  << ", " << entries[i].length() << " chars): ";
        for (std::size_t j = 0; j < entries[i].length(); j++) {
            std::cout << entries[i][j];
        }
        std::cout << std::endl;
    }

    return 0;
}
