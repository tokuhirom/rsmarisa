use marisa::grimoire::trie::entry::Entry;
use marisa::grimoire::vector::vector::Vector;

fn main() {
    let words = vec![
        "a",
        "app",
        "apple",
        "application",
        "apply",
        "banana",
        "band",
    ];

    let mut entries: Vector<Entry> = Vector::new();

    for word in &words {
        let mut entry = Entry::new();
        entry.set_str(word.as_bytes());
        entries.push_back(entry);
    }

    println!("Before sort:");
    for i in 0..entries.size() {
        let e = entries[i];
        print!(
            "  {} ({} chars): ",
            std::str::from_utf8(e.as_bytes()).unwrap(),
            e.length()
        );
        for j in 0..e.length() {
            print!("{}", e.get(j) as char);
        }
        println!();
    }

    // Set IDs
    for i in 0..entries.size() {
        entries[i].set_id(i);
    }

    // Sort
    let entries_slice = entries.as_mut_slice();
    marisa::grimoire::algorithm::sort::sort(entries_slice);

    println!("\nAfter sort:");
    for i in 0..entries.size() {
        let e = entries[i];
        print!(
            "  {} (id={}, {} chars): ",
            std::str::from_utf8(e.as_bytes()).unwrap(),
            e.id(),
            e.length()
        );
        for j in 0..e.length() {
            print!("{}", e.get(j) as char);
        }
        println!();
    }
}
