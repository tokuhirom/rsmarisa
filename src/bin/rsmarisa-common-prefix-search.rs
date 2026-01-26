//! rsmarisa-common-prefix-search - Find common prefixes in a MARISA trie
//!
//! For each query, finds all keys that are prefixes of the query.

use clap::Parser;
use marisa::{Agent, Keyset, Trie};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "rsmarisa-common-prefix-search")]
#[command(about = "Find keys that are prefixes of given queries")]
#[command(version)]
struct Args {
    /// Maximum number of results to show (0 = no limit)
    #[arg(short = 'n', long, default_value = "10")]
    max_num_results: usize,

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
    let mut keyset = Keyset::new();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("error: failed to read query: {}", e);
                process::exit(30);
            }
        };

        agent.set_query_str(&line);

        // Collect all matches
        while trie.common_prefix_search(&mut agent) {
            keyset.push_back_key(agent.key());
        }

        if keyset.empty() {
            if let Err(e) = writeln!(stdout, "not found") {
                eprintln!("error: failed to write output: {}", e);
                process::exit(31);
            }
        } else {
            let count = keyset.size();
            if let Err(e) = writeln!(stdout, "{} found", count) {
                eprintln!("error: failed to write output: {}", e);
                process::exit(31);
            }

            let max_results = if args.max_num_results == 0 {
                count
            } else {
                args.max_num_results.min(count)
            };

            for i in 0..max_results {
                let key = keyset.get(i);
                let key_str = key.as_str();
                if let Err(e) = writeln!(stdout, "{}\t{}\t{}", key.id(), key_str, line) {
                    eprintln!("error: failed to write output: {}", e);
                    process::exit(31);
                }
            }
        }

        keyset.reset();
    }
}
