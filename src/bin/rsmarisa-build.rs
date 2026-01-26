//! rsmarisa-build - Build MARISA trie dictionaries
//!
//! Reads keys from standard input or files and builds a trie dictionary.

use clap::Parser;
use marisa::{Keyset, Trie};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "rsmarisa-build")]
#[command(about = "Build a MARISA trie dictionary from text input")]
#[command(version)]
struct Args {
    /// Number of tries [1-127] (default: 3)
    #[arg(short = 'n', long, value_name = "N", default_value = "3")]
    num_tries: u32,

    /// Build with text TAIL (default)
    #[arg(short = 't', long, conflicts_with = "binary_tail")]
    text_tail: bool,

    /// Build with binary TAIL
    #[arg(short = 'b', long)]
    binary_tail: bool,

    /// Arrange siblings in weight order (default)
    #[arg(short = 'w', long, conflicts_with = "label_order")]
    weight_order: bool,

    /// Arrange siblings in label order
    #[arg(short = 'l', long)]
    label_order: bool,

    /// Cache level [1-5] (default: 3)
    #[arg(short = 'c', long, value_name = "N", default_value = "3")]
    cache_level: u32,

    /// Output file (default: stdout)
    #[arg(short = 'o', long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Input files (default: stdin)
    files: Vec<PathBuf>,
}

fn read_keys<R: BufRead>(input: R, keyset: &mut Keyset) -> io::Result<()> {
    for line in input.lines() {
        let line = line?;

        // Check for tab-delimited weight
        if let Some(delim_pos) = line.rfind('\t') {
            let key = &line[..delim_pos];
            let weight_str = &line[delim_pos + 1..];

            if let Ok(weight) = weight_str.parse::<f32>() {
                keyset.push_back_bytes(key.as_bytes(), weight)?;
                continue;
            }
        }

        // No weight or invalid weight - use default weight of 1.0
        keyset.push_back_str(&line)?;
    }
    Ok(())
}

fn build_config(args: &Args) -> i32 {
    use marisa::base::{CacheLevel, NodeOrder, TailMode};

    let mut config = 0i32;

    // Num tries
    config |= (args.num_tries & 0x7F) as i32;

    // Cache level
    let cache = match args.cache_level {
        1 => CacheLevel::Tiny,
        2 => CacheLevel::Small,
        3 => CacheLevel::Normal,
        4 => CacheLevel::Large,
        5 => CacheLevel::Huge,
        _ => {
            eprintln!("error: cache level must be 1-5");
            process::exit(1);
        }
    };
    config |= cache as i32;

    // Tail mode
    let tail_mode = if args.binary_tail {
        TailMode::BinaryTail
    } else {
        TailMode::TextTail
    };
    config |= tail_mode as i32;

    // Node order
    let node_order = if args.label_order {
        NodeOrder::Label
    } else {
        NodeOrder::Weight
    };
    config |= node_order as i32;

    config
}

fn main() {
    let args = Args::parse();

    // Validate num_tries
    if args.num_tries < 1 || args.num_tries > 127 {
        eprintln!("error: num-tries must be in range [1, 127]");
        process::exit(1);
    }

    let mut keyset = Keyset::new();

    // Read keys from stdin or files
    if args.files.is_empty() {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin.lock());
        if let Err(e) = read_keys(reader, &mut keyset) {
            eprintln!("error: failed to read keys from stdin: {}", e);
            process::exit(10);
        }
    } else {
        for path in &args.files {
            match File::open(path) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    if let Err(e) = read_keys(reader, &mut keyset) {
                        eprintln!("error: failed to read keys from {}: {}", path.display(), e);
                        process::exit(11);
                    }
                }
                Err(e) => {
                    eprintln!("error: failed to open {}: {}", path.display(), e);
                    process::exit(12);
                }
            }
        }
    }

    // Build trie
    let mut trie = Trie::new();
    let config = build_config(&args);

    trie.build(&mut keyset, config);

    // Print statistics to stderr
    eprintln!("#keys: {}", trie.num_keys());
    eprintln!("#nodes: {}", trie.num_nodes());
    eprintln!("size: {}", trie.io_size());

    // Save or write to stdout
    if let Some(output_path) = args.output {
        if let Err(e) = trie.save(output_path.to_str().unwrap()) {
            eprintln!("error: failed to save dictionary to {}: {}", output_path.display(), e);
            process::exit(30);
        }
    } else {
        // Write to stdout (write directly to the writer)
        use marisa::grimoire::io::Writer;
        let _stdout = io::stdout();
        let _writer = Writer::new();

        // Trie::write is internal, use the public API via save to a temp file then copy
        // For now, require -o option
        eprintln!("error: stdout output not yet supported, please use -o option");
        process::exit(31);
    }
}
