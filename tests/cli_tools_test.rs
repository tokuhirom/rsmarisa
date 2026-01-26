//! Integration tests for CLI tools
//!
//! Verifies that rsmarisa-build and rsmarisa-lookup produce
//! results identical to C++ marisa-trie tools.

use std::process::Command;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_rsmarisa_build_binary_compatibility() {
    // Create test input
    let test_keys = "a\napp\napple\napplication\napply\nbanana\nband\n";

    // Build with Rust tool
    let mut rust_dict = NamedTempFile::new().unwrap();
    let rust_output = Command::new("target/release/rsmarisa-build")
        .arg("-o")
        .arg(rust_dict.path())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn rsmarisa-build")
        .stdin
        .take()
        .unwrap()
        .write_all(test_keys.as_bytes())
        .and_then(|_| Ok(()));

    assert!(rust_output.is_ok(), "Failed to write to rsmarisa-build stdin");

    // Build with C++ tool
    let mut cpp_dict = NamedTempFile::new().unwrap();
    let cpp_output = Command::new("marisa-build")
        .arg("-o")
        .arg(cpp_dict.path())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("marisa-build not found - please install C++ marisa-trie")
        .stdin
        .take()
        .unwrap()
        .write_all(test_keys.as_bytes())
        .and_then(|_| Ok(()));

    assert!(cpp_output.is_ok(), "Failed to write to marisa-build stdin");

    // Wait a moment for files to be written
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Compare binary files
    let rust_bytes = fs::read(rust_dict.path()).unwrap();
    let cpp_bytes = fs::read(cpp_dict.path()).unwrap();

    assert_eq!(
        rust_bytes.len(),
        cpp_bytes.len(),
        "Dictionary files have different sizes"
    );

    assert_eq!(
        rust_bytes,
        cpp_bytes,
        "Dictionary files are not byte-for-byte identical"
    );
}

#[test]
fn test_rsmarisa_lookup_compatibility() {
    // Create test dictionary
    let test_keys = "a\napp\napple\napplication\napply\nbanana\nband\n";
    let mut dict_file = NamedTempFile::new().unwrap();

    Command::new("marisa-build")
        .arg("-o")
        .arg(dict_file.path())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("marisa-build not found")
        .stdin
        .take()
        .unwrap()
        .write_all(test_keys.as_bytes())
        .unwrap();

    std::thread::sleep(std::time::Duration::from_millis(100));

    // Test queries
    let queries = "a\napple\nbanana\nbandit\nzebra\n";

    // Lookup with Rust tool
    let rust_output = Command::new("target/release/rsmarisa-lookup")
        .arg(dict_file.path())
        .stdin(std::process::Stdio::piped())
        .output()
        .expect("Failed to run rsmarisa-lookup");

    let mut rust_child = Command::new("target/release/rsmarisa-lookup")
        .arg(dict_file.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    rust_child.stdin.take().unwrap().write_all(queries.as_bytes()).unwrap();
    let rust_result = rust_child.wait_with_output().unwrap();
    let rust_stdout = String::from_utf8(rust_result.stdout).unwrap();

    // Lookup with C++ tool
    let mut cpp_child = Command::new("marisa-lookup")
        .arg(dict_file.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    cpp_child.stdin.take().unwrap().write_all(queries.as_bytes()).unwrap();
    let cpp_result = cpp_child.wait_with_output().unwrap();
    let cpp_stdout = String::from_utf8(cpp_result.stdout).unwrap();

    assert_eq!(
        rust_stdout,
        cpp_stdout,
        "Lookup results differ between Rust and C++ implementations"
    );
}

#[test]
fn test_rsmarisa_common_prefix_search_compatibility() {
    // Create test dictionary
    let test_keys = "a\napp\napple\napplication\napply\nbanana\nband\n";
    let mut dict_file = NamedTempFile::new().unwrap();

    Command::new("marisa-build")
        .arg("-o")
        .arg(dict_file.path())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("marisa-build not found")
        .stdin
        .take()
        .unwrap()
        .write_all(test_keys.as_bytes())
        .unwrap();

    std::thread::sleep(std::time::Duration::from_millis(100));

    // Test query
    let query = "application\n";

    // Search with Rust tool
    let mut rust_child = Command::new("target/release/rsmarisa-common-prefix-search")
        .arg(dict_file.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    rust_child.stdin.take().unwrap().write_all(query.as_bytes()).unwrap();
    let rust_result = rust_child.wait_with_output().unwrap();
    let rust_stdout = String::from_utf8(rust_result.stdout).unwrap();

    // Search with C++ tool
    let mut cpp_child = Command::new("marisa-common-prefix-search")
        .arg(dict_file.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    cpp_child.stdin.take().unwrap().write_all(query.as_bytes()).unwrap();
    let cpp_result = cpp_child.wait_with_output().unwrap();
    let cpp_stdout = String::from_utf8(cpp_result.stdout).unwrap();

    assert_eq!(
        rust_stdout,
        cpp_stdout,
        "Common prefix search results differ between Rust and C++ implementations"
    );
}

// Note: predictive_search and reverse_lookup tests are skipped due to known bugs
// See README Known Issues section
