use marisa::grimoire::trie::entry::Entry;
use marisa::grimoire::trie::tail::{Tail, TailMode};
use marisa::grimoire::vector::vector::Vector;

fn main() {
    let words = vec![
        "a", "app", "apple", "application", "apply", "banana", "band"
    ];

    let mut entries: Vector<Entry> = Vector::new();

    for word in &words {
        let mut entry = Entry::new();
        entry.set_str(word.as_bytes());
        entries.push_back(entry);
    }

    // Set IDs and sort
    for i in 0..entries.size() {
        entries[i].set_id(i);
    }
    let entries_slice = entries.as_mut_slice();
    marisa::grimoire::algorithm::sort::sort(entries_slice);

    println!("Sorted entries (processing in reverse):");
    for i in (0..entries.size()).rev() {
        let e = entries[i];
        let word = std::str::from_utf8(e.as_bytes()).unwrap();
        println!("  {} (id={})", word, e.id());
    }

    // Build tail    let mut tail = Tail::new();
    let mut offsets: Vector<u32> = Vector::new();
    tail.build(&mut entries, &mut offsets, TailMode::TextTail);

    println!("\nTail buffer (cannot access directly - it's private)");

    println!("Offsets:");
    for i in 0..offsets.size() {
        println!("  words[{}] ({}) -> offset {}", i, words[i], offsets[i]);
    }
}
