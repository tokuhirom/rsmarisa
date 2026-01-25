//! Binary compatibility test with C++ marisa-trie
//!
//! This test verifies that Rust-generated binary files are identical to
//! C++-generated files when built from the same keyset.

use marisa::{Keyset, Trie};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

#[test]
#[ignore] // Run manually with: cargo test binary_compat -- --ignored
fn test_binary_compatibility_with_cpp() {
    // Create temporary directory for test files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let rust_file = temp_dir.path().join("rust_output.marisa");
    let cpp_file = temp_dir.path().join("cpp_output.marisa");

    // Build Rust trie
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

    println!("Rust trie stats:");
    println!("  Keys: {}", trie.num_keys());
    println!("  Nodes: {}", trie.num_nodes());
    println!("  I/O size: {} bytes", trie.io_size());

    // Save Rust trie
    trie.save(rust_file.to_str().unwrap())
        .expect("Failed to save Rust trie");

    // Build C++ trie if tools are available
    let cpp_test_exe = find_cpp_test_binary();
    if let Some(cpp_exe) = cpp_test_exe {
        println!("Running C++ test program: {}", cpp_exe.display());
        let output = Command::new(&cpp_exe)
            .arg(cpp_file.to_str().unwrap())
            .output()
            .expect("Failed to run C++ test program");

        if !output.status.success() {
            panic!(
                "C++ test program failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        println!("{}", String::from_utf8_lossy(&output.stdout));

        // Compare files
        let rust_data = fs::read(&rust_file).expect("Failed to read Rust file");
        let cpp_data = fs::read(&cpp_file).expect("Failed to read C++ file");

        println!("\nFile sizes:");
        println!("  Rust: {} bytes", rust_data.len());
        println!("  C++:  {} bytes", cpp_data.len());

        if rust_data == cpp_data {
            println!("\n✓ Binary files are identical!");
        } else {
            println!("\n✗ Binary files differ!");

            // Show first difference
            for (i, (r, c)) in rust_data.iter().zip(cpp_data.iter()).enumerate() {
                if r != c {
                    println!("First difference at byte {}: Rust=0x{:02x}, C++=0x{:02x}", i, r, c);
                    break;
                }
            }

            if rust_data.len() != cpp_data.len() {
                println!("Length difference: {} bytes",
                    (rust_data.len() as i64 - cpp_data.len() as i64).abs());
            }

            panic!("Binary compatibility test failed");
        }
    } else {
        println!("C++ test binary not found - skipping comparison");
        println!("To enable full test:");
        println!("1. Build C++ marisa-trie");
        println!("2. Compile tests/cpp_binary_test.cc");
        println!("3. Place it in target/debug/ or target/release/");
    }
}

fn find_cpp_test_binary() -> Option<PathBuf> {
    // Look for cpp_test in build directories
    let candidates = vec![
        PathBuf::from("target/debug/cpp_test"),
        PathBuf::from("target/release/cpp_test"),
        PathBuf::from("/tmp/cpp_test"),
    ];

    for path in candidates {
        if path.exists() {
            return Some(path);
        }
    }

    None
}
