use rust_marisa::keyset::Keyset;
use rust_marisa::trie::Trie;

fn main() {
    let mut keyset = Keyset::new();
    keyset.push_back_str("a").unwrap();
    keyset.push_back_str("app").unwrap();

    println!("Building trie with [\"a\", \"app\"]...");
    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Trie built successfully");
    println!("  num_keys: {}", trie.num_keys());

    // Access internals to print tail buffer (this is just for debugging)
    println!("\nThis would require adding debug methods to inspect tail buffer and offsets");
    println!("For now, let's just compare with C++ output");
}
