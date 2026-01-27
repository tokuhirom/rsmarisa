//! Debug tool to track sizes written by each component

use rsmarisa::{Keyset, Trie};
use std::io::{self, Cursor, Write};

// Custom writer that tracks bytes written
struct TrackingWriter {
    inner: Cursor<Vec<u8>>,
    total_written: usize,
    component_sizes: Vec<(String, usize)>,
}

impl TrackingWriter {
    fn new() -> Self {
        Self {
            inner: Cursor::new(Vec::new()),
            total_written: 0,
            component_sizes: Vec::new(),
        }
    }

    fn mark_component(&mut self, name: &str) {
        let current_pos = self.inner.position() as usize;
        let size = current_pos - self.total_written;
        if size > 0 {
            self.component_sizes.push((name.to_string(), size));
        }
        self.total_written = current_pos;
    }

    fn report(&self) {
        println!("Component sizes:");
        let mut total = 0;
        for (name, size) in &self.component_sizes {
            println!("  {}: {} bytes", name, size);
            total += size;
        }
        println!("  Total: {} bytes", total);
    }

    fn into_inner(self) -> Vec<u8> {
        self.inner.into_inner()
    }
}

impl Write for TrackingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

fn main() {
    let mut keyset = Keyset::new();
    let words = vec![
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
    ];

    for word in &words {
        keyset.push_back_str(word).unwrap();
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    println!("Trie stats:");
    println!("  Keys: {}", trie.num_keys());
    println!("  Nodes: {}", trie.num_nodes());
    println!("  I/O size: {} bytes", trie.io_size());
    println!();

    // For now just write to file normally
    trie.save("tmp/debug_output.marisa")
        .expect("Failed to save");

    let size = std::fs::metadata("tmp/debug_output.marisa")
        .expect("Failed to get metadata")
        .len();
    println!("Written to tmp/debug_output.marisa: {} bytes", size);
}
