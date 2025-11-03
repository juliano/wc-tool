use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn ccwc_file(opt: &str) -> Vec<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/test.txt");
    let mut command = Command::new(env!("CARGO_BIN_EXE_ccwc"));
    if !opt.is_empty() {
        command.arg(opt);
    }
    command.arg(path);
    let output = command.output().expect("Failed to execute ccwc");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    return stdout
        .trim()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
}

fn ccwc_stdin(opt: &str) -> Vec<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/test.txt");
    let file: File = File::open(&path).expect("Failed to open test file");

    let mut command = Command::new(env!("CARGO_BIN_EXE_ccwc"));
    if !opt.is_empty() {
        command.arg(opt);
    }
    let output = command
        .stdin(Stdio::from(file))
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to execute ccwc");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    return stdout
        .trim()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
}

#[test]
fn test_ccwc_without_options() {
    let parts = ccwc_file("");
    assert_eq!(
        parts.len(),
        4,
        "Expected 4 parts: lines words bytes filename"
    );

    let lines: u64 = parts[0].parse().expect("lines should be a number");
    let words: u64 = parts[1].parse().expect("words should be a number");
    let bytes: u64 = parts[2].parse().expect("bytes should be a number");

    assert_eq!(lines, 7145);
    assert_eq!(words, 58164);
    assert_eq!(bytes, 342190);
}

#[test]
fn test_ccwc_with_option_c() {
    let parts = ccwc_file("-c");

    assert_eq!(parts.len(), 2, "Expected 2 parts: bytes filename");
    let bytes: u64 = parts[0].parse().expect("bytes should be a number");
    assert_eq!(bytes, 342190);
}

#[test]
fn test_ccwc_with_option_l() {
    let parts = ccwc_file("-l");

    assert_eq!(parts.len(), 2, "Expected 2 parts: lines filename");

    let lines: u64 = parts[0].parse().expect("lines should be a number");
    assert_eq!(lines, 7145);
}

#[test]
fn test_ccwc_with_option_w() {
    let parts = ccwc_file("-w");

    assert_eq!(parts.len(), 2, "Expected 2 parts: words filename");

    let words: u64 = parts[0].parse().expect("words should be a number");
    assert_eq!(words, 58164);
}

#[test]
fn test_ccwc_with_option_m() {
    let parts = ccwc_file("-m");

    assert_eq!(parts.len(), 2, "Expected 2 parts: chars filename");

    let chars: u64 = parts[0].parse().expect("chars should be a number");
    assert_eq!(chars, 339292);
}

#[test]
fn test_ccwc_stdin_default() {
    let parts = ccwc_stdin("");

    assert_eq!(parts.len(), 3, "Expected 3 parts: lines words bytes");

    let lines: u64 = parts[0].parse().expect("lines should be a number");
    let words: u64 = parts[1].parse().expect("words should be a number");
    let bytes: u64 = parts[2].parse().expect("bytes should be a number");

    assert_eq!(lines, 7145);
    assert_eq!(words, 58164);
    assert_eq!(bytes, 342190);
}

#[test]
fn test_ccwc_stdin_option_l() {
    let parts = ccwc_stdin("-l");

    assert_eq!(parts.len(), 1, "Expected 1 part: lines");

    let lines: u64 = parts[0].parse().expect("lines should be a number");
    assert_eq!(lines, 7145);
}

#[test]
fn test_ccwc_stdin_option_w() {
    let parts = ccwc_stdin("-w");

    assert_eq!(parts.len(), 1, "Expected 1 part: words");

    let words: u64 = parts[0].parse().expect("words should be a number");
    assert_eq!(words, 58164);
}

#[test]
fn test_ccwc_stdin_option_c() {
    let parts = ccwc_stdin("-c");

    assert_eq!(parts.len(), 1, "Expected 1 part: bytes");

    let bytes: u64 = parts[0].parse().expect("bytes should be a number");
    assert_eq!(bytes, 342190);
}

#[test]
fn test_ccwc_stdin_option_m() {
    let parts = ccwc_stdin("-m");

    assert_eq!(parts.len(), 1, "Expected 1 part: chars");

    let chars: u64 = parts[0].parse().expect("chars should be a number");
    assert_eq!(chars, 339292);
}
