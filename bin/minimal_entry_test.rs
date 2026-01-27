use rsmarisa::grimoire::trie::entry::{Entry, StringComparer};

fn main() {
    // Test two simple words    let word1 = "app";
    let word2 = "apple";

    let mut e1 = Entry::new();
    e1.set_str(word1.as_bytes());

    let mut e2 = Entry::new();
    e2.set_str(word2.as_bytes());

    println!("Comparing '{}' vs '{}':", word1, word2);
    println!("Entry bytes (forward): '{}' = {:?}", word1, word1.as_bytes());
    println!("Entry bytes (forward): '{}' = {:?}", word2, word2.as_bytes());
    println!();

    println!("Entry::get() (reverse access):");
    print!("  '{}' -> ", word1);
    for i in 0..e1.length() {
        print!("{:02x} ", e1.get(i));
    }
    println!();

    print!("  '{}' -> ", word2);
    for i in 0..e2.length() {
        print!("{:02x} ", e2.get(i));
    }
    println!();
    println!();

    // Test StringComparer
    let cmp_result = StringComparer::compare(&e1, &e2);
    println!("StringComparer::compare(e1, e2) = {} (true means e1 > e2)", cmp_result);

    // Manual comparison
    println!("\nManual comparison:");
    for i in 0..e1.length().max(e2.length()) {
        if i >= e1.length() {
            println!("  [{}]: e1 ended, e2 has more -> e1 < e2");
            break;
        } else if i >= e2.length() {
            println!("  [{}]: e2 ended, e1 has more -> e1 > e2");
            break;
        } else {
            let b1 = e1.get(i);
            let b2 = e2.get(i);
            println!("  [{}]: {:02x} vs {:02x} ({})", i, b1, b2,
                if b1 == b2 { "equal" } else if b1 > b2 { "e1 > e2" } else { "e1 < e2" });
            if b1 != b2 {
                break;
            }
        }
    }
}
