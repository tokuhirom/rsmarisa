//! Benchmark binary for perf profiling.
//!
//! This is NOT a criterion benchmark. It is a simple binary designed to be used with
//! `perf record` / `perf report` to identify hot spots in rsmarisa.
//!
//! Usage:
//!   cargo build --release --example bench
//!   ./target/release/examples/bench
//!
//! Profiling:
//!   perf record -g ./target/release/examples/bench
//!   perf report

use rsmarisa::{Agent, Keyset, Trie};
use std::hint::black_box;
use std::time::Instant;

/// Number of iterations for each benchmark operation.
const ITERATIONS: usize = 1000;

/// Hiragana syllables used as building blocks.
const SYLLABLES: &[&str] = &[
    "あ", "い", "う", "え", "お", "か", "き", "く", "け", "こ", "さ", "し", "す", "せ", "そ",
    "た", "ち", "つ", "て", "と", "な", "に", "ぬ", "ね", "の", "は", "ひ", "ふ", "へ", "ほ",
    "ま", "み", "む", "め", "も", "や", "ゆ", "よ", "ら", "り", "る", "れ", "ろ", "わ", "を",
    "ん", "が", "ぎ", "ぐ", "げ",
];

/// Kanji/surface forms paired with readings.
const SURFACES: &[&str] = &[
    "亜", "位", "宇", "絵", "尾", "火", "木", "空", "毛", "子", "左", "市", "酢", "背", "祖",
    "田", "地", "津", "手", "戸", "名", "荷", "布", "根", "野", "葉", "日", "風", "辺", "帆",
    "間", "実", "無", "目", "物", "矢", "湯", "世", "良", "理", "留", "礼", "路", "和", "尾",
    "運", "雅", "義", "具", "下",
];

/// Generate "読み\t表層形" format keys for predictive_search, lookup, reverse_lookup.
///
/// Mimics akaza's kana-kanji dictionary where keys are "reading\tsurface".
fn generate_dict_keys() -> Vec<String> {
    let mut keys = Vec::new();

    for (i, &s1) in SYLLABLES.iter().enumerate() {
        // Length 1: "あ\t亜"
        keys.push(format!("{}\t{}", s1, SURFACES[i % SURFACES.len()]));

        for (j, &s2) in SYLLABLES.iter().enumerate() {
            // Length 2: "あい\t亜位"
            let reading2 = format!("{}{}", s1, s2);
            let surface2 = format!(
                "{}{}",
                SURFACES[i % SURFACES.len()],
                SURFACES[j % SURFACES.len()]
            );
            keys.push(format!("{}\t{}", reading2, surface2));

            // Length 3 (subset): "あいう\t亜位宇/あいう"
            if j < 4 {
                for (k, &s3) in SYLLABLES.iter().enumerate().take(4) {
                    let reading3 = format!("{}{}{}", s1, s2, s3);
                    let surface3 = format!(
                        "{}{}{}",
                        SURFACES[i % SURFACES.len()],
                        SURFACES[j % SURFACES.len()],
                        SURFACES[k % SURFACES.len()]
                    );
                    keys.push(format!("{}\t{}/{}", reading3, surface3, reading3));
                }
            }
        }
    }

    keys
}

/// Generate kana-only keys for common_prefix_search.
///
/// Mimics akaza's kana trie where keys are pure hiragana readings of varying length.
/// Shorter readings are prefixes of longer ones, which is the typical pattern
/// for common_prefix_search.
fn generate_kana_keys() -> Vec<String> {
    let mut keys = Vec::new();

    for &s1 in SYLLABLES {
        // Length 1: "あ"
        keys.push(s1.to_string());

        for &s2 in SYLLABLES {
            // Length 2: "あい"
            keys.push(format!("{}{}", s1, s2));
        }
    }

    // Add some longer keys (length 3-5) for a subset
    for &s1 in &SYLLABLES[..10] {
        for &s2 in &SYLLABLES[..10] {
            for &s3 in &SYLLABLES[..5] {
                keys.push(format!("{}{}{}", s1, s2, s3));
            }
        }
    }

    keys
}

fn bench_build(label: &str, keys: &[String]) -> Trie {
    let start = Instant::now();

    let mut keyset = Keyset::new();
    for key in keys {
        keyset.push_back_str(key).unwrap();
    }

    let mut trie = Trie::new();
    trie.build(&mut keyset, 0);

    let elapsed = start.elapsed();
    eprintln!(
        "build({}):  {:>8.2} ms  ({} keys, {} nodes, {} bytes)",
        label,
        elapsed.as_secs_f64() * 1000.0,
        trie.num_keys(),
        trie.num_nodes(),
        trie.total_size(),
    );
    trie
}

fn bench_predictive_search(trie: &Trie, prefixes: &[&str]) {
    let start = Instant::now();
    let mut total_results = 0usize;

    for _ in 0..ITERATIONS {
        for &prefix in prefixes {
            let mut agent = Agent::new();
            agent.set_query_str(prefix);
            while trie.predictive_search(&mut agent) {
                black_box(agent.key().as_bytes());
                black_box(agent.key().id());
                total_results += 1;
            }
        }
    }

    let elapsed = start.elapsed();
    eprintln!(
        "predictive_search:      {:>8.2} ms  ({} iters x {} prefixes, {} total results)",
        elapsed.as_secs_f64() * 1000.0,
        ITERATIONS,
        prefixes.len(),
        total_results,
    );
}

fn bench_common_prefix_search(trie: &Trie, queries: &[&str]) {
    let start = Instant::now();
    let mut total_results = 0usize;

    for _ in 0..ITERATIONS {
        for &query in queries {
            let mut agent = Agent::new();
            agent.set_query_str(query);
            while trie.common_prefix_search(&mut agent) {
                black_box(agent.key().as_bytes());
                black_box(agent.key().id());
                total_results += 1;
            }
        }
    }

    let elapsed = start.elapsed();
    eprintln!(
        "common_prefix_search:   {:>8.2} ms  ({} iters x {} queries, {} total results)",
        elapsed.as_secs_f64() * 1000.0,
        ITERATIONS,
        queries.len(),
        total_results,
    );
}

fn bench_lookup(trie: &Trie, keys: &[String]) {
    let start = Instant::now();
    let mut found = 0usize;

    for _ in 0..ITERATIONS {
        for key in keys {
            let mut agent = Agent::new();
            agent.set_query_str(key);
            if trie.lookup(&mut agent) {
                black_box(agent.key().id());
                found += 1;
            }
        }
    }

    let elapsed = start.elapsed();
    eprintln!(
        "lookup:                 {:>8.2} ms  ({} iters x {} keys, {} found)",
        elapsed.as_secs_f64() * 1000.0,
        ITERATIONS,
        keys.len(),
        found,
    );
}

fn bench_reverse_lookup(trie: &Trie, num_keys: usize) {
    let start = Instant::now();

    for _ in 0..ITERATIONS {
        for id in 0..num_keys {
            let mut agent = Agent::new();
            agent.set_query_id(id);
            trie.reverse_lookup(&mut agent);
            black_box(agent.key().as_bytes());
        }
    }

    let elapsed = start.elapsed();
    eprintln!(
        "reverse_lookup:         {:>8.2} ms  ({} iters x {} ids)",
        elapsed.as_secs_f64() * 1000.0,
        ITERATIONS,
        num_keys,
    );
}

fn main() {
    eprintln!("=== rsmarisa bench (for perf profiling) ===\n");

    // 1. Generate data and build tries
    let dict_keys = generate_dict_keys();
    let kana_keys = generate_kana_keys();
    eprintln!(
        "Generated {} dict keys, {} kana keys\n",
        dict_keys.len(),
        kana_keys.len()
    );

    let dict_trie = bench_build("dict", &dict_keys);
    let kana_trie = bench_build("kana", &kana_keys);
    let num_dict_keys = dict_trie.num_keys();

    // 2. Prepare search queries mimicking akaza patterns

    // Short prefixes for predictive_search (like "reading\t" prefix matching)
    let predictive_prefixes = [
        "あ",
        "か",
        "さ",
        "た",
        "あい",
        "かき",
        "さし",
        "あ\t",
        "か\t",
        "さし\t",
    ];

    // Long kana strings for common_prefix_search (like segmenting a sentence)
    let common_prefix_queries = [
        "あいうえおかきくけこ",
        "さしすせそたちつてと",
        "なにぬねのはひふへほ",
        "まみむめもやゆよらり",
        "がぎぐげごわをん",
    ];

    // 3. Run benchmarks
    eprintln!();
    bench_predictive_search(&dict_trie, &predictive_prefixes);
    bench_common_prefix_search(&kana_trie, &common_prefix_queries);
    bench_lookup(&dict_trie, &dict_keys);
    bench_reverse_lookup(&dict_trie, num_dict_keys);

    eprintln!("\nDone.");
}
