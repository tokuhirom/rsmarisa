//! rsmarisa-dump - Dump all keys from a MARISA trie dictionary
//!
//! Extracts and prints all keys stored in a dictionary.

use clap::Parser;
use marisa::{Agent, Trie};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "rsmarisa-dump")]
#[command(about = "Dump all keys from a MARISA trie dictionary")]
#[command(version)]
struct Args {
    /// Delimiter for output (default: newline)
    #[arg(short = 'd', long, default_value = "\n")]
    delimiter: String,

    /// Use memory-mapped I/O (not yet implemented, always uses read)
    #[arg(short = 'm', long)]
    mmap_dictionary: bool,

    /// Read entire dictionary into memory (default)
    #[arg(short = 'r', long)]
    read_dictionary: bool,

    /// Dictionary files (or stdin if none specified)
    dictionaries: Vec<PathBuf>,
}

fn dump_trie(trie: &Trie, delimiter: &str) -> io::Result<()> {
    let mut num_keys = 0;
    let mut agent = Agent::new();
    let mut stdout = io::stdout();

    // Set empty query to get all keys via predictive search
    agent.set_query_str("");

    loop {
        if trie.predictive_search(&mut agent) {
            // Found a key
            let key_bytes = agent.key().as_bytes();
            stdout.write_all(key_bytes)?;
            stdout.write_all(delimiter.as_bytes())?;
            num_keys += 1;
        } else {
            // No more keys
            break;
        }
    }

    eprintln!("#keys: {}", num_keys);
    Ok(())
}

fn dump_file(path: Option<&PathBuf>, args: &Args) -> io::Result<()> {
    let mut trie = Trie::new();

    if args.mmap_dictionary {
        eprintln!("warning: memory-mapped I/O not yet implemented, using read instead");
    }

    if let Some(path) = path {
        eprintln!("input: {}", path.display());
        trie.load(path.to_str().unwrap())?;
    } else {
        eprintln!("input: <stdin>");
        // For stdin, we need to use Reader
        // For now, require a file argument
        eprintln!("error: reading from stdin not yet supported, please specify a file");
        return Err(io::Error::new(io::ErrorKind::Other, "stdin not supported"));
    }

    dump_trie(&trie, &args.delimiter)
}

fn main() {
    let args = Args::parse();

    let result = if args.dictionaries.is_empty() {
        // Read from stdin
        dump_file(None, &args)
    } else {
        // Process each dictionary file
        let mut last_result = Ok(());
        for path in &args.dictionaries {
            if let Err(e) = dump_file(Some(path), &args) {
                eprintln!("error: failed to dump {}: {}", path.display(), e);
                last_result = Err(e);
                break;
            }
        }
        last_result
    };

    if let Err(_) = result {
        process::exit(20);
    }
}
