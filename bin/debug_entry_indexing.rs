use marisa::grimoire::trie::entry::Entry;

fn main() {
    let s = "application";
    let mut entry = Entry::new();
    entry.set_str(s.as_ptr(), s.len());

    println!("String: \"{}\" (length={})", s, s.len());
    println!("\nEntry indexing (reverse order):");
    for i in 0..entry.length() {
        let c = entry.get(i);
        println!("  entry.get({:2}) = '{}'", i, c as char);
    }

    println!("\nC++ style loop: for j in 1..=length, push current[length - j]");
    for j in 1..=entry.length() {
        let c = entry.get(entry.length() - j);
        print!("{}", c as char);
    }
    println!();

    println!("\nRust current code: for j in 0..length, push current.get(length - 1 - j)");
    for j in 0..entry.length() {
        let c = entry.get(entry.length() - 1 - j);
        print!("{}", c as char);
    }
    println!();
}
