//! Base module tests.
//!
//! Ported from: tests/base-test.cc

use marisa::base::*;

#[test]
#[allow(clippy::assertions_on_constants)]
fn test_word_size() {
    // Word size should be either 32 or 64
    assert!(WORD_SIZE == 32 || WORD_SIZE == 64);

    #[cfg(target_pointer_width = "64")]
    assert_eq!(WORD_SIZE, 64);

    #[cfg(target_pointer_width = "32")]
    assert_eq!(WORD_SIZE, 32);
}

#[test]
fn test_invalid_constants() {
    assert_eq!(INVALID_LINK_ID, u32::MAX);
    assert_eq!(INVALID_KEY_ID, u32::MAX);
    assert_eq!(INVALID_EXTRA, u32::MAX >> 8);
}

#[test]
fn test_error_code() {
    let err = ErrorCode::Ok;
    assert_eq!(err as i32, 0);

    let err = ErrorCode::StateError;
    assert_eq!(err as i32, 1);

    let err = ErrorCode::IoError;
    assert_eq!(err as i32, 9);
}

#[test]
fn test_default_config_values() {
    assert_eq!(CacheLevel::default(), CacheLevel::Normal);
    assert_eq!(TailMode::default(), TailMode::TextTail);
    assert_eq!(NodeOrder::default(), NodeOrder::Weight);
}

#[test]
fn test_num_tries_constants() {
    assert_eq!(NumTries::MIN, 0x00001);
    assert_eq!(NumTries::MAX, 0x0007F);
    assert_eq!(NumTries::DEFAULT, 0x00003);
}

// TODO: Port more tests from base-test.cc as implementation progresses
