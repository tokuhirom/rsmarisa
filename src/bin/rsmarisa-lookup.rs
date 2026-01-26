//! rsmarisa-lookup - Look up keys in a MARISA trie dictionary
//!
//! Reads queries from standard input and looks them up in the dictionary.

use clap::Parser;
use marisa::{Agent, Trie};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "rsmarisa-lookup")]
#[command(about = "Look up keys in a MARISA trie dictionary")]
#[command(version)]
struct Args {
    /// Use memory-mapped I/O (not yet implemented, always uses read)
    #[arg(short = 'm', long)]
    mmap_dictionary: bool,

    /// Read entire dictionary into memory (default)
    #[arg(short = 'r', long)]
    read_dictionary: bool,

    /// Dictionary file
    dictionary: PathBuf,
}

fn main() {
    let args = Args::parse();

    if args.mmap_dictionary {
        eprintln!("warning: memory-mapped I/O not yet implemented, using read instead");
    }

    // Load dictionary
    let mut trie = Trie::new();
    if let Err(e) = trie.load(args.dictionary.to_str().unwrap()) {
        eprintln!(
            "error: failed to load dictionary from {}: {}",
            args.dictionary.display(),
            e
        );
        process::exit(20);
    }

    // Process queries from stdin
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut agent = Agent::new();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("error: failed to read query: {}", e);
                process::exit(30);
            }
        };

        agent.set_query_str(&line);

        if trie.lookup(&mut agent) {
            // Found - output id and key
            let key_id = agent.key().id();
            if let Err(e) = writeln!(stdout, "{}\t{}", key_id, line) {
                eprintln!("error: failed to write output: {}", e);
                process::exit(31);
            }
        } else {
            // Not found - output -1
            if let Err(e) = writeln!(stdout, "-1\t{}", line) {
                eprintln!("error: failed to write output: {}", e);
                process::exit(31);
            }
        }
    }
}
