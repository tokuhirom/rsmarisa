use rsmarisa::grimoire::trie::entry::{Entry, StringComparer};

fn main() {
    let mut a = Entry::new();
    a.set_str("a".as_bytes());

    let mut banana = Entry::new();
    banana.set_str("banana".as_bytes());

    println!("a vs banana:");
    println!(
        "  compare(a, banana) = {}",
        StringComparer::compare(&a, &banana)
    );
    println!(
        "  compare(banana, a) = {}",
        StringComparer::compare(&banana, &a)
    );

    let mut app = Entry::new();
    app.set_str("app".as_bytes());

    let mut apply = Entry::new();
    apply.set_str("apply".as_bytes());

    println!("\napp vs apply:");
    println!(
        "  compare(app, apply) = {}",
        StringComparer::compare(&app, &apply)
    );
    println!(
        "  compare(apply, app) = {}",
        StringComparer::compare(&apply, &app)
    );

    println!("\napp.get(0) = '{}'", app.get(0) as char);
    println!("apply.get(0) = '{}'", apply.get(0) as char);
    println!("app.get(1) = '{}'", app.get(1) as char);
    println!("apply.get(1) = '{}'", apply.get(1) as char);
}
