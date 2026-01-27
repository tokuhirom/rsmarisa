//! rsmarisa-reverse-lookup - Restore keys from IDs in a MARISA trie
//!
//! Reads key IDs from standard input and outputs the corresponding keys.

use clap::Parser;
use rsmarisa::{Agent, Trie};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "rsmarisa-reverse-lookup")]
#[command(about = "Restore keys from their IDs")]
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

    // Process key IDs from stdin
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut agent = Agent::new();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("error: failed to read input: {}", e);
                process::exit(30);
            }
        };

        let key_id: usize = match line.trim().parse() {
            Ok(id) => id,
            Err(e) => {
                eprintln!("error: invalid key ID '{}': {}", line, e);
                process::exit(30);
            }
        };

        agent.set_query_id(key_id);

        trie.reverse_lookup(&mut agent);

        let key = agent.key();
        let key_str = key.as_str();
        if let Err(e) = writeln!(stdout, "{}\t{}", key.id(), key_str) {
            eprintln!("error: failed to write output: {}", e);
            process::exit(31);
        }
    }
}
