#include <marisa/grimoire/trie/cache.h>
#include <iostream>
#include <iomanip>

using namespace marisa::grimoire::trie;

int main() {
    std::cout << "Testing Cache::set_base and extra()" << std::endl;

    Cache cache;
    std::cout << "Initial state:" << std::endl;
    std::cout << "  link (hex): 0x" << std::hex << std::setw(8) << std::setfill('0') << cache.link() << std::dec << std::endl;
    std::cout << "  base: " << (int)cache.base() << std::endl;
    std::cout << "  extra: " << cache.extra() << std::endl;

    cache.set_base(0xFF);
    std::cout << "\nAfter set_base(0xFF):" << std::endl;
    std::cout << "  link (hex): 0x" << std::hex << std::setw(8) << std::setfill('0') << cache.link() << std::dec << std::endl;
    std::cout << "  base: " << (int)cache.base() << std::endl;
    std::cout << "  extra: " << cache.extra() << std::endl;

    if (cache.base() == 0xFF && cache.extra() == 0) {
        std::cout << "\n✓ Test PASSED" << std::endl;
        return 0;
    } else {
        std::cout << "\n✗ Test FAILED" << std::endl;
        std::cout << "  Expected: base=255, extra=0" << std::endl;
        std::cout << "  Got: base=" << (int)cache.base() << ", extra=" << cache.extra() << std::endl;
        return 1;
    }
}
